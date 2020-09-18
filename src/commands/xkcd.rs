use crate::util;
use crate::XkcdState;
use crate::XKCD_INDEX_PATH;
use serde::Deserialize;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct Comic {
    month: String,
    num: u64,
    link: String,
    year: String,
    news: String,
    safe_title: String,
    transcript: String,
    alt: String,
    img: String,
    title: String,
    day: String,
}

lazy_static::lazy_static! {
    static ref XKCD_INDEX: tantivy::Index = Index::open_in_dir(&*XKCD_INDEX_PATH)
        .expect("could not open xkcd index");
}

lazy_static::lazy_static! {
    static ref XKCD_INDEX_READER: tantivy::IndexReader = XKCD_INDEX
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into().expect("could not create reader for xkcd index");
}

lazy_static::lazy_static! {
    static ref XKCD_INDEX_SCHEMA: tantivy::schema::Schema = {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("alt", TEXT | STORED);
        schema_builder.add_text_field("img", STORED);
        schema_builder.add_u64_field("number", IntOptions::default().set_stored().set_indexed());

        schema_builder.build()
    };
}

#[command]
#[description = "Post the xkcd comic specified"]
#[example = "547"]
#[num_args(1)]
pub async fn xkcd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let xkcd_query = args.rest();

    // let reqwest_client = &util::get_reqwest_client(&ctx).await?;
    // try to load index
    let searcher = XKCD_INDEX_READER.searcher();

    let number = XKCD_INDEX_SCHEMA.get_field("number").unwrap();
    let title = XKCD_INDEX_SCHEMA.get_field("title").unwrap();
    let alt = XKCD_INDEX_SCHEMA.get_field("alt").unwrap();
    let img = XKCD_INDEX_SCHEMA.get_field("img").unwrap();

    let query_parser = QueryParser::for_index(&*XKCD_INDEX, vec![title, alt]);

    let mut top_docs = {
        let query = query_parser
            .parse_query(&xkcd_query)
            .map_err(|e| format!("Failed to parse query: {:?}", e))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(1))
            .map_err(|e| format!("failed index search"))?;

        top_docs
    };

    let (_, doc_address) = top_docs.pop().ok_or("nothing found")?;

    let retrieved_doc = searcher
        .doc(doc_address)
        .map_err(|e| format!("doc does not exist: {:?}", e))?;

    let title = retrieved_doc.get_first(title).unwrap().text().unwrap();
    let alt = retrieved_doc.get_first(alt).unwrap().text().unwrap();
    let img = retrieved_doc.get_first(img).unwrap().text().unwrap();
    let number = retrieved_doc.get_first(number).unwrap().u64_value();

    let xkcd_link = format!("https://xkcd.com/{}", number);

    match msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.author(|a| a.name("Xkcd"))
                    .title(&title)
                    .description(&alt)
                    .color((0, 120, 220))
                    .image(&img)
                    .url(&xkcd_link)
                    .footer(|f| f.text(&xkcd_link))
            })
        })
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending message: {:?}", e);
            Err(e.into())
        }
    }
}

#[command]
#[owners_only]
pub async fn index_xkcd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let indexed = ctx
        .data
        .read()
        .await
        .get::<XkcdState>()
        .ok_or("Failed to access xkcd state")?
        .indexed;

    let reqwest_client = ctx
        .data
        .read()
        .await
        .get::<crate::ReqwestClient>()
        .ok_or("Failed to get reqwest client")?
        .clone();

    let index = if XKCD_INDEX_PATH.is_dir() {
        // Use existing index
        Index::open_in_dir(&*XKCD_INDEX_PATH)
            .map_err(|e| format!("Failed to load tantivy index: {}", e))?
    } else {
        // create index from scratch
        tokio::fs::create_dir_all(&*XKCD_INDEX_PATH).await?;
        Index::create_in_dir(&*XKCD_INDEX_PATH, XKCD_INDEX_SCHEMA.clone())
            .map_err(|e| format!("Failed to create tantivy index: {}", e))?
    };

    let mut index_writer = index
        .writer(50_000_000)
        .map_err(|e| format!("failed to create index writer: {:?}", e))?;

    let title = XKCD_INDEX_SCHEMA.get_field("title").unwrap();
    let alt = XKCD_INDEX_SCHEMA.get_field("alt").unwrap();
    let img = XKCD_INDEX_SCHEMA.get_field("img").unwrap();
    let number = XKCD_INDEX_SCHEMA.get_field("number").unwrap();

    let newest_comic: Comic = reqwest_client
        .get("https://xkcd.com/info.0.json")
        .send()
        .await?
        .json()
        .await?;

    for i in indexed..=newest_comic.num {
        if i == 404 {
            continue;
        }
        let comic: Comic = reqwest_client
            .get(&format!("https://xkcd.com/{}/info.0.json", i))
            .send()
            .await?
            .json()
            .await?;

        index_writer.add_document(doc!(
            title => comic.title,
            alt => comic.alt,
            img => comic.img,
            number => comic.num,
        ));

        // report progess every 100 comics
        if i % 100 == 0 {
            let _ = msg
                .channel_id
                .say(
                    &ctx,
                    &format!("Progess on xkcd indexing: {} of {}", i, newest_comic.num),
                )
                .await;
        }

        // be nice on the api usage
        tokio::time::delay_for(std::time::Duration::from_millis(500)).await;
        if i % 5 == 0 {
            tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
        }
        if i % 100 == 0 {
            tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
        }
    }

    {
        let mut write = ctx.data.write().await;
        let index_state = write
            .get_mut::<crate::XkcdState>()
            .ok_or("failed to acces xkcd index state")?;
        index_state.indexed = newest_comic.num + 1;
        index_state.save();
    }

    index_writer
        .commit()
        .map_err(|e| format!("Failed to commit changes to index: {:?}", e))?;

    Ok(())
}
