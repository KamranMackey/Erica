use crate::models::tmdb::movie::*;
use crate::utilities::format_int;

use humantime::format_duration;

use isolang::Language;

use itertools::Itertools;

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::redirect::Policy;

use serde::Deserialize;

use serenity::client::Context;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;

use serenity::model::prelude::Message;

use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchMovie>
}

#[derive(Debug, Deserialize)]
pub struct SearchMovie {
    pub id: u64 // The movie's ID from the search result, which is all we need.
}

#[command]
#[aliases("film")]
#[description("Gets detailed information about a movie from The Movie Database.")]
pub fn movie(context: &mut Context, message: &Message, arguments: Args) -> CommandResult {
    if arguments.rest().is_empty() {
        message.channel_id.send_message(&context, |message| {
            message.embed(|embed| {
                embed.title("Error: Invalid movie title provided.");
                embed.description("You have provided an invalid movie title. Please try again.");
                embed.color(0x00FF_0000)
            })
        })?;
        return Ok(());
    }

    let mut movie: String = arguments.rest().to_string();

    let api_key = crate::config::tmdb_key().expect("Could not find API key for The Movie Database...").to_owned();
    let user_agent: &str = concat!(env!("CARGO_PKG_NAME"), ", v", env!("CARGO_PKG_VERSION"));
    let client = Client::builder().user_agent(user_agent).redirect(Policy::none()).build()?;

    let search_endpoint = "https://api.themoviedb.org/3/search/movie";
    let search_response: RequestBuilder;

    // This is a pretty hacky way of being able to search by year, but
    // surprisingly enough it actually works from what I've tested, and
    // while it might be a tad slow, it should compute fast enough to not
    // make users wonder why its taking so long for the response to send.
    //
    // This code follows the y: notation syntax as available on the website
    // for The Movie Database, with the additional ability to use year: in
    // place of y:, depending on the user's preference.
    if movie.contains("y:") || movie.contains("year:") {
        movie = movie.replace(" y:", "").replace(" year:", "");
        let mut year_rev: Vec<char> = movie.chars().rev().take(4).collect();
        year_rev.reverse();
        let year = year_rev.iter().map(|c| c).join("");
        movie = movie.replace(&year, "");
        search_response = client.get(search_endpoint).query(&[("api_key", &api_key), ("query", &movie), ("year", &year)]);
    } else {
        search_response = client.get(search_endpoint).query(&[("api_key", &api_key), ("query", &movie)]);
    }

    let search_result: SearchResponse = search_response.send()?.json()?;
    let search_results = search_result.results;

    if search_results.is_empty() {
        message.channel_id.send_message(&context, |message| {
            message.content(format!(
                "Sorry, I was unable to find a movie on TMDb matching the term `{}`. \
                Please try a different search term.",
                movie
            ))
        })?;
        return Ok(());
    }

    let movie_id = search_results.first().unwrap().id;
    let movie_endpoint = format!("https://api.themoviedb.org/3/movie/{}", movie_id);
    let movie_response = client.get(&movie_endpoint).query(&[("api_key", &api_key)]).send()?;
    let movie_result: Movie = movie_response.json()?;

    let movie_tagline = match movie_result.tagline {
        Some(tagline) => {
            if tagline.is_empty() {
                "".to_string()
            } else {
                format!("*{}*", tagline)
            }
        }
        None => "".to_string()
    };

    let movie_overview = match movie_result.overview {
        Some(overview) => {
            if !movie_tagline.is_empty() {
                format!("\n\n{}", overview)
            } else {
                overview
            }
        }
        None => "".to_string()
    };

    let movie_studios = if movie_result.production_companies.is_empty() {
        "No Known Studios".to_string()
    } else {
        movie_result.production_companies.iter().map(|c| &c.name).join("\n")
    };

    let movie_collection = match movie_result.belongs_to_collection {
        Some(collection) => collection.name,
        None => "N/A".to_string()
    };

    let movie_homepage = match movie_result.homepage {
        Some(homepage) => format!("[Website]({})", homepage),
        None => "No Website".to_string()
    };

    let movie_id = movie_result.id.to_string();
    let movie_title = movie_result.title.as_str();
    let movie_status = movie_result.status;
    let movie_language = Language::from_639_1(&movie_result.original_language).unwrap().to_name().to_string();
    let movie_release_date = movie_result.release_date.unwrap().format("%B %e, %Y").to_string();
    let movie_budget = format_int(movie_result.budget as usize);
    let movie_revenue = format_int(movie_result.revenue as usize);
    let movie_imdb = format!("[IMDb](https://www.imdb.com/title/{})", movie_result.imdb_id.unwrap());
    let movie_url = format!("https://www.themoviedb.org/movie/{}", movie_id);
    let movie_genres = movie_result.genres.iter().map(|g| &g.name).join("\n");
    let movie_popularity = format!("{}%", movie_result.popularity);
    let movie_poster_uri = movie_result.poster_path.unwrap();
    let movie_poster = format!("https://image.tmdb.org/t/p/original/{}", &movie_poster_uri.replace("/", ""));
    let movie_user_score = format!("{}/100", movie_result.vote_average * 10.0);
    let movie_user_score_count = movie_result.vote_count;
    let movie_runtime = format_duration(Duration::from_secs(movie_result.runtime.unwrap() * 60)).to_string();
    let movie_external_links = format!("{} | {}", movie_homepage, movie_imdb);

    message.channel_id.send_message(&context, |message| {
        message.embed(|embed| {
            embed.title(movie_title);
            embed.url(movie_url);
            embed.color(0x0001_d277);
            embed.thumbnail(movie_poster);
            embed.description(format!("{}{}", movie_tagline, movie_overview));
            embed.fields(vec![
                ("Status", movie_status, true),
                ("TMDb ID", movie_id, true),
                ("Language", movie_language, true),
                ("Runtime", movie_runtime, true),
                ("Release Date", movie_release_date, true),
                ("Collection", movie_collection, true),
                ("Popularity", movie_popularity, true),
                ("User Score", format!("{} ({} votes)", movie_user_score, movie_user_score_count), true),
                ("Budget", format!("${}", movie_budget), true),
                ("Box Office", format!("${}", movie_revenue), true),
                ("Genres", movie_genres, true),
                ("Studios", movie_studios, true),
                ("External Links", movie_external_links, false),
            ]);
            embed.footer(|footer| footer.text("Powered by the The Movie Database API."))
        })
    })?;

    Ok(())
}
