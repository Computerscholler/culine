use scraper::{Html, Selector};
use serde_json::{Value};

mod recipe;
use recipe::{Recipe};

#[derive(Debug)]
enum Error {
    NoLDData,
    JSONParseError { orig_error: serde_json::Error },
    RequestError { orig_error: reqwest::Error},
    NoRecipeFound,
}

async fn get_webpage_body(url: &str) -> Result<String, reqwest::Error> {
    match reqwest::get(url).await {
        Ok(resp) => resp.text().await,
        Err(err) => Err(err)
    }
} 

fn extract_json_ld_from_html(html: Html) -> Result<Vec<String>, Error> {
    // If parse fails the selector is wrong so we can unwrap here
    let json_ld_selector = Selector::parse("script[type=\"application/ld+json\"]").unwrap();
    let selected = html.select(&json_ld_selector);

    let mut json_ld_fields: Vec<String> = vec![];

    for element in selected {
        json_ld_fields.push(element.inner_html())
    }

    if json_ld_fields.len() == 0 {
        Err(Error::NoLDData)
    } else {
        Ok(json_ld_fields)

    }
}

fn parse_json_to_recipe(raw_json_ld: &str) -> Result<Recipe, Error>{
    let val = match serde_json::from_str::<Value>(raw_json_ld) {
        Ok(val) => val,
        Err(err) => return Err(Error::JSONParseError { orig_error: err })
    };
    
    if let Some(graph) = val["@graph"].as_array() {
        for obj in graph.iter() {
            if obj["@type"] == "Recipe" {
                match serde_json::from_value::<Recipe>(obj.clone()) {
                    Ok(recipe) => return Ok(recipe),
                    Err(err) => return Err(Error::JSONParseError { orig_error: err })
                }
            }
        }
    } else {
        if val["@type"] == "Recipe" {
            match serde_json::from_value::<Recipe>(val) {
                Ok(recipe) => return Ok(recipe),
                Err(err) => return Err(Error::JSONParseError { orig_error: err })
            }
        }
    }

    return Err(Error::NoRecipeFound)
}

async fn parse_recipe_from_url(url: &str) -> Result<Recipe, Error> {
    match get_webpage_body(url).await {
        Ok(body) => {
            let html: Html = Html::parse_document(body.as_str());
            match extract_json_ld_from_html(html) {
                Ok(raw_json_ld_vec) => {
                    for raw_json_ld in raw_json_ld_vec {
                        match parse_json_to_recipe(raw_json_ld.as_str()) {
                            Ok(recipe) => return Ok(recipe),
                            Err(Error::NoRecipeFound) => continue,
                            Err(error) => return Err(error)
                        }
                    }
                    return Err(Error::NoRecipeFound)
                },
                Err(err) => Err(err)
            }
        },
        Err(err) => Err(Error::RequestError { orig_error: err })
    } 
}

#[tokio::main]
async fn main() {
    let recipe = parse_recipe_from_url("https://www.recipetineats.com/easy-yeast-bread-recipe-no-knead/").await.unwrap();
    println!("{:?}", recipe)
}