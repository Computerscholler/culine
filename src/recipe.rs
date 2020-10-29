use serde::{Deserialize, Serialize, Deserializer};
use serde_json::{Value};
use serde::de::{self, Visitor, SeqAccess, MapAccess};
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Nutrition {
    pub calories: Option<String>,
    pub servingSize: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub name: String,
}

#[derive(Debug, Serialize, Default)]
pub struct Instruction {
    pub text: String,
    pub image: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub video: Option<String>
}

#[derive(Debug, Serialize, Default)]
pub struct Author {
    pub name: Option<String>,
    pub url: Option<String>,
    #[serde(rename(deserialize = "@id"))]
    pub id: Option<String>
}

#[derive(Debug, Serialize)]
pub struct VecOrString(Vec<String>);

#[derive(Debug, Serialize)]
pub struct Image(Vec<String>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    #[serde(rename(deserialize = "@id"))]
    pub id: Option<String>,
    pub description: Option<String>,
    pub image: Option<Image>,
    #[serde(rename(deserialize = "cookTime"))]
    pub cook_time: Option<String>,
    #[serde(rename(deserialize = "prepTime"))]
    pub prep_time: Option<String>,
    pub nutrition: Option<Nutrition>,
    #[serde(rename(deserialize = "recipeYield"))]
    pub recipe_yield: Option<VecOrString>,
    #[serde(rename(deserialize = "recipeInstructions"))]
    pub instructions: Vec<Instruction>,
    #[serde(rename(deserialize = "recipeIngredient"))]
    pub ingredients: Vec<String>,
    #[serde(rename(deserialize = "recipeCategory"))]
    pub category: Option<VecOrString>,
    pub author: Option<Author>,
    #[serde(rename(deserialize = "totalTime"))]
    pub total_time: Option<String>,
    pub keywords: Option<String>,
    pub video: Option<Video>
}

// A Visitor for entries that may be a vec or a string; returns vec for all
struct VecOrStringVisitor;
impl<'de> Visitor<'de> for VecOrStringVisitor {
    type Value = VecOrString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string, an object or an array")
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E> {
        Ok(VecOrString(vec![s]))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> 
    where
        A: SeqAccess<'de> {
        let mut vec: Vec<String> = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Ok(Some(elem)) = seq.next_element::<String>() {
            vec.push(elem)
        }

        Ok(VecOrString(vec))
    }
}

impl<'de> Deserialize<'de> for VecOrString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de> {
        deserializer.deserialize_any(VecOrStringVisitor)
    }
}

struct AuthorVisitor;
impl<'de> Visitor<'de> for AuthorVisitor {
    type Value = Author;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string, an object or an array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> 
    where
        A: SeqAccess<'de> {
        if let Ok(Some(map)) = seq.next_element::<HashMap<String, String>>() {
            let mut author = Author { ..Default::default() };
            for (k, v) in map {
                match k.as_str() {
                    "@id" => author.id = Some(v.clone()),
                    "name" => author.name = Some(v.clone()),
                    "url" => author.url = Some(v.clone()),
                    &_ => {},
                }
            };
            Ok(author)
        } else {
            Ok(Author { ..Default::default()})
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de> {
        let mut author = Author { ..Default::default() };
        while let Ok(Some((key, value))) = map.next_entry::<String, String>() {
            match key.as_str() {
                "@id" => author.id = Some(value),
                "name" => author.name = Some(value),
                "url" => author.url = Some(value),
                &_ => continue,
            }
        }
        Ok(author)
    }
}

impl<'de> Deserialize<'de> for Author {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de> {
        deserializer.deserialize_any(AuthorVisitor)
    }
}

struct ImageVisitor;
impl<'de> Visitor<'de> for ImageVisitor {
    type Value = Image;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string, an object or an array")
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E> {
        Ok(Image(vec![s]))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> 
    where
        A: SeqAccess<'de> {
        let mut vec: Vec<String> = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Ok(Some(elem)) = seq.next_element::<String>() {
            vec.push(elem)
        }

        Ok(Image(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de> {
        
        while let Ok(Some((key, value))) = map.next_entry::<String, String>() {
            if key.as_str() == "@id" {
                return Ok(Image(vec![value]))
            }
        }

        Ok(Image(vec![]))
    }
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de> {
        deserializer.deserialize_any(ImageVisitor)
    }
}

struct InstructionVisitor;
impl<'de> Visitor<'de> for InstructionVisitor {
    type Value = Instruction;
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string or HowToStep")
    }

    fn visit_string<E>(self, s: String) -> Result<Instruction, E> {
        Ok(Instruction { text: s, ..Default::default() })
    }

    fn visit_map<A>(self, mut access: A) -> Result<Instruction, A::Error> 
    where
        A: MapAccess<'de> {
            let mut instruction = Instruction { ..Default::default() };
            while let Some((key, value)) = access.next_entry::<String, String>()? {
                match key.as_str() {
                    "@type" => continue, // TODO: handle @type
                    "text" => instruction.text = value,
                    "name" => instruction.name = Some(value),
                    "image" => instruction.image = Some(value),
                    "url" => instruction.url = Some(value),
                    &_ => continue
                }
            }
            Ok(instruction)
    }
}


impl<'de> Deserialize<'de> for Instruction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de> {
        deserializer.deserialize_any(InstructionVisitor)
    }
}
