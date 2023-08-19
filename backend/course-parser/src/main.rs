use std::fs;
use tl;
use tl::VDom;
const DATA_DIR: &str = "../../data";
const PAGE_DIR: &str = "../../data/pages";

fn get_course_filenames(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut filenames: Vec<String> = Vec::new();

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_file() {
            let file_name = entry.file_name();
            filenames.push(file_name.to_string_lossy().to_string());
        }
    }

    Ok(filenames)
}

struct Course {
    id: String,
}

// this function returns a Result type
fn parse_course(html: &str) -> Result<Course, Box<dyn std::error::Error>> {
    let dom = tl::parse(html, tl::ParserOptions::default())?;
    let content = dom.get_element_by_id("content");

    // if there is no content element, we assume it is a new course
    if content.is_some() {
        return parse_old_course(&dom);
    }

    // 558 courses are new
    Err("Unknown course html format".into())
}

fn parse_old_course(dom: &VDom) -> Result<Course, Box<dyn std::error::Error>> {
    // find all div class="panel-body" elements and assert that there is only one
    let mut panel_bodies = dom.get_elements_by_class_name("panel-body");
    let parser = dom.parser();
    let mut valid_bodies = 0;
    for panel_body in panel_bodies {
        let resulting = panel_body.get(parser).unwrap().as_tag().unwrap();
        let dls = resulting.query_selector(parser, "dl").unwrap();
        for handle in dls {
            let node = handle.get(parser).unwrap().as_tag().unwrap();
            // print the first 50 characters of the inner text
            println!("{}", node.inner_text(parser)[..51].to_string());
            valid_bodies += 1;
        }
    }
    if valid_bodies == 1 {
        return Ok(Course {
            id: "new course".to_string(),
        });
    } else if valid_bodies > 1 {
        return Err("Multiple panel-bodies found".into());
    }

    Err("dl was not found inside of a panel-body".into())
}

fn try_parsing(
    html: &str,
    parser_fn: fn(&str) -> Result<Course, Box<dyn std::error::Error>>,
) -> bool {
    match parser_fn(html) {
        Ok(_) => true,
        Err(err) => {
            eprintln!("Error: {}", err);
            false
        }
    }
}

fn main() {
    // time how long it takes to run this
    let start = std::time::Instant::now();
    match get_course_filenames(PAGE_DIR) {
        Ok(filenames) => {
            let mut fails = 0;
            let mut passes = 0;
            for filename in filenames {
                let path = format!("{}/{}", PAGE_DIR, filename);
                let html = fs::read_to_string(path).unwrap();
                if try_parsing(&html, parse_course) {
                    passes += 1;
                } else {
                    fails += 1;
                    println!("Failed to parse {}", filename);
                }
            }
            println!(
                "{} passes, {} fails\nparsed: {:.0}%",
                passes,
                fails,
                passes as f64 / (passes + fails) as f64 * 100.0
            );
        }
        Err(err) => eprintln!("Error: {}", err),
    }
    println!("Time elapsed: {:.2?}", start.elapsed());
}
