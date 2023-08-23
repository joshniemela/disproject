use tl::VDom;
use eyre::Result;

///////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURE
///////////////////////////////////////////////////////////////////////////////
#[allow(dead_code)]
pub struct Course {
    id: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct CourseInformation {
    id: String,
    ects: f32,
    block: Vec<Block>,
    schedule: Vec<Schedule>,
    language: Language,
    duration: Duration,
    degree: Vec<Degree>,
    capacity: u32,
}

#[derive(Debug)] enum Block {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

#[derive(Debug)]
enum Schedule {
    A,
    B,
    C,
    D,
}

#[derive(Debug)]
enum Language {
    Danish,
    English,
}

#[derive(Debug, Eq, PartialEq)]
enum Duration {
    One = 1,
    Two = 2,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
enum Degree {
    Phd,
    Bachelor,
    Master,
}

///////////////////////////////////////////////////////////////////////////////
// LOGIC
///////////////////////////////////////////////////////////////////////////////

/// Parses html file.
///
/// Main entrypoint, and the function that gets called in main.rs.
///
/// # Parameters
/// * `html: &str` - `&str` representation of the contents of an html file
///
/// # Errors
/// Bubbles up the error resulting from any of functions called internally.
pub fn parse_course(html: &str) -> Result<Course, Box<dyn std::error::Error>> {
    let dom = tl::parse(html, tl::ParserOptions::default())?;
    let content = dom.get_element_by_id("content");

    // if there is no content element, we assume it is a new course
    if content.is_some() {
        return parse_course_info(&dom);
    }

    Err("Unknown course html format".into())
}

fn parse_course_info(dom: &VDom) -> Result<Course, Box<dyn std::error::Error>> {
    // find all div class="panel-body" elements and assert that there is only one
    let panel_bodies = dom.get_elements_by_class_name("panel-body");
    let parser = dom.parser();

    // there might be multiple panel-bodies, so we need to check each one
    // for the dl element (only the course info should have a dl element)
    for (_i, panel_body) in panel_bodies.enumerate() {
        let mut dl_elements = panel_body
            .get(parser)
            .ok_or("Failed to get panel-body")?
            .as_tag()
            .ok_or("Failed to get panel-body as tag")?
            .query_selector(parser, "dl")
            .ok_or("Failed to get dl from panel-body")?;
        match dl_elements.next() {
            Some(handle) => {
                let node = handle
                    .get(parser)
                    .ok_or("Failed to get node")?
                    .as_tag()
                    .ok_or("Failed to get node as tag")?;
                // print the first 50 characters of the inner text
                //println!("{}", node.inner_text(parser)[..51].to_string());
                //println!("panel-body {}", i);
                //println!("dl: {}", node.inner_text(parser));
                // parse DL
                let course_infos = parse_dl(node, parser)?;
                println!("{course_infos:?}");
                // parse the course information
                let coerced_course_info = coerce_course_info(course_infos)?;
                println!("{coerced_course_info:?}");
                return Err("Not implemented".into());
            }
            None => continue,
        }
    }
    Err("No dl element found in the panel-body".into())
}

// return a list of tuples of (key, value)
fn parse_dl(
    dl_tag: &tl::HTMLTag,
    parser: &tl::Parser,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut result: Vec<(String, String)> = Vec::new();
    let children = dl_tag.children();
    // for even numbers, we expect a dt element, odd numbers we expect a dd element
    // make a pair of precisely two strings
    let mut pair: Vec<String> = Vec::with_capacity(2);
    for (_i, child) in children.top().iter().enumerate() {
        let node = child
            .get(parser)
            .ok_or("Failed to get node whilst parsing dl")?;
        match node.as_tag() {
            Some(tag) => {
                if tag.name() == "dt" {
                    pair.push(node.inner_text(parser).to_string());
                } else if tag.name() == "dd" {
                    pair.push(node.inner_text(parser).to_string());
                    if pair.len() == 2 {
                        result.push((pair[0].clone(), pair[1].clone()));
                        pair.clear();
                    }
                } else {
                    return Err("Expected dt or dd element".into());
                }
            }
            None => continue,
        }
    }
    // if the pair is not empty then we have had an odd number of elements and should error
    if !pair.is_empty() {
        return Err("Odd number of elements in dl".into());
    }
    Ok(result)
}


fn parse_language(language: &str) -> Result<Language, Box<dyn std::error::Error>> {
    // println!("Language information passed in: {language}");
    match language {
        "English" => Ok(Language::English),
        "Dansk" => Ok(Language::Danish),
        _ => Err(format!("Unknown language ({language})").into()),
    }
}

fn parse_ects(ects: &str) -> Result<f32, Box<dyn std::error::Error>> {
    // expect to find either "15 ECTS" or "7.5 ECTS" in the string
    let ects = ects
        .chars()
        .take_while(|c| c.is_numeric() || c == &'.')
        .collect::<String>()
        // rename the potential error to something more meaningful
        .parse::<f32>()
        .map_err(|e| format!("Failed to parse ects ({e})"))?;

    Ok(ects)
}

#[allow(dead_code)]
fn parse_degree(degree: &str) -> Result<Vec<Degree>, Box<dyn std::error::Error>> {
    let mut result: Vec<Degree> = Vec::new();
    // Loop through the degree string and find all the degrees
    // Look for either "Master", "Bachelor", "Kandidat" or "Ph.d."

    let alphabetic_degree = degree
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();

    // loop through a 2 character sliding window and deal with the fact that they might not be alphabetic
    for i in 0..alphabetic_degree.len() - 1 {
        let sliding_window = &alphabetic_degree[i..i + 2];
        match sliding_window {
            "Ba" => result.push(Degree::Bachelor),
            "Ma" | "Ka" => result.push(Degree::Master),
            "Ph" => result.push(Degree::Phd),
            _ => continue,
        }
    }

    // print if it was phd
    if result.contains(&Degree::Phd) {
        return Err(format!("Phd course found ({degree})").into());
    }

    // Sort and deduplicate
    result.sort();
    result.dedup();
    if result.is_empty() {
        return Err("No degree found".into());
    }
    Ok(result)
}

fn parse_capacity(capacity: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // find the first number and parse it
    let capacity = capacity
        .chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>()
        .parse::<u32>()
        .map_err(|e| format!("Failed to parse capacity ({e})"))?;
    Ok(capacity)
}

fn parse_schedule(schedule: &str) -> Result<Vec<Schedule>, Box<dyn std::error::Error>> {
    // println!("Schedule info passed in: {schedule}");
    let mut schedule_vec: Vec<Schedule> = Vec::new();
    
    if schedule.contains("A") {
        schedule_vec.push(Schedule::A);
    }

    if schedule.contains("B") {
        schedule_vec.push(Schedule::B);
    }

    if schedule.contains("C") {
        schedule_vec.push(Schedule::C);
    }

    if schedule.contains("D") {
        schedule_vec.push(Schedule::D);
    }

    if schedule_vec.len() > 0 {
        Ok(schedule_vec)
    } else {
        Err("Unknown schedule".into())
    }
}

fn parse_block(block: &str) -> Result<Vec<Block>, Box<dyn std::error::Error>> {
    let mut blocks: Vec<Block> = Vec::new();

    if block.contains("Blok 1") {
        blocks.push(Block::One);
    }

    if block.contains("Blok 2") {
        blocks.push(Block::Two);
    }

    if block.contains("Blok 3") {
        blocks.push(Block::Three);
    }

    if block.contains("Blok 4") {
        blocks.push(Block::Four);
    }

    if block.contains("Blok 5") {
        blocks.push(Block::Five);
    }

    if blocks.len() > 0 {
        Ok(blocks)
    } else {
        Err("Unknown block".into())
    }
}

fn parse_duration(duration: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    // either 1 blo(c)k, 2 blo(c)ks or 1 semester
    // grab the first 3 chars
    let chopped_duration = duration.chars().take(3).collect::<String>();
    match chopped_duration.as_str() {
        "1 b" => Ok(Duration::One),
        "2 b" | "1 s" => Ok(Duration::Two),
        _ => Err("Unknown duration".into()),
    }
}

fn coerce_course_info(
    course_info: Vec<(String, String)>,
) -> Result<CourseInformation, Box<dyn std::error::Error>> {

    dbg!(&course_info);
    let mut id: Option<String> = None;
    let mut ects: Option<f32> = None;
    let mut block: Option<Vec<Block>> = None;
    let mut schedule: Option<Vec<Schedule>> = None;
    let mut language: Option<Language> = None;
    let mut duration: Option<Duration> = None;
    let mut degree: Option<Vec<Degree>> = None;
    let mut capacity: Option<u32> = None;
    
    for (key, value) in &course_info {
        // first iterate through only to find the block, since  this will tell us if we
        // are dealing with the faculty of science (they use blocks) or the other faculties
        // Check the first 5 chars of the key to see if it is "Place"
        let chopped_key = key.chars().take(5).collect::<String>();
        if chopped_key == "Place" {
            block = Some(parse_block(value)?);
        }
    }

    for (key, value) in course_info {
        match key.as_str() {
            "Language" | "Sprog" => language = Some(parse_language(&value)?),
            "Kursuskode" | "Course code" => id = Some(value), // "Kursuskode" is the danish version of "Course code
            "Point" | "Credit" => ects = Some(parse_ects(&value)?), // "Point" is the danish version of "Credit"
            "Level" => degree = Some(parse_degree(&value)?),
            "Duration" => duration = Some(parse_duration(&value)?),
            "Schedule" | "Skemagruppe" => schedule = Some(parse_schedule(&value)?),
            "Course capacity" => capacity = Some(parse_capacity(&value)?),
            _ => continue,
        }
    }
    // print every error with the contents of the course_info
    let id = id.ok_or("Failed to get id")?;
    let ects = ects.ok_or("Failed to get ects")?;
    let block = block.ok_or("Failed to get block")?;
    let schedule = schedule.ok_or("Failed to get schedule")?;
    let language = language.ok_or("Failed to get language")?;
    let duration = duration.ok_or("Failed to get duration")?;
    let degree = degree.ok_or("Failed to get degree")?;
    println!("{id}: {degree:?}");
    let capacity = capacity.ok_or("Failed to get capacity")?;

    Ok(CourseInformation {
        id,
        ects,
        block,
        schedule,
        language,
        duration,
        degree,
        capacity,
    })
}