#![allow(unused_variables)]
#![warn(dead_code)]

use std::{
    collections::HashMap, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, vec
};
use serde::Serialize;
use eval::Expr;

struct EndPoint {
    page: String,
    renderer: fn(page: &str) -> String,
}

struct Template<T> where T: Serialize {
    name: &'static str,
    value: T,
} 

fn main() {
    let mut routes: HashMap<String, EndPoint> = HashMap::new();

    let mut static_folders: Vec<String> = Vec::new();

    register_static_folder("html/static", &mut static_folders);

    register_end_point(&mut routes, "404", "html/404.ftml", |page: &str| {
        render::<String>(page, None)
    });
    
    register_end_point(&mut routes, "/", "html/index.html", |page: &str| {

        let mut object = HashMap::new();
        object.insert("foos", vec!["Hello", "world", "!"]);

        let temp = Template {
            name: "obj",
            value: object,
        };
        let temp2 = vec![temp];

        render(page, Some(temp2))
        
    });



    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &mut routes, &static_folders);
    }
}

fn handle_connection(mut stream: TcpStream, routes: &mut HashMap<String, EndPoint>, static_folders: &Vec<String>) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();


    if http_request.len() == 0 {
        return println!("No request found");
    }


    match get_protocol(&http_request[0]) {
        "GET" => println!("Request: {:#?}", http_request[0]),
        _ => return println!("Unknown request"),
    }
    let end_point = get_end_point(&http_request[0]);
    let page = get_page(&routes, end_point, &static_folders);



    let status_line = "HTTP/1.1 200 OK";
    let length = page.len();

    let response = format!(
        "{status_line}\r\nServer: FredWork/0.1.0\r\nContent-Length: {length}\r\n\r\n{page}",
    );

    stream.write_all(response.as_bytes()).unwrap();

    
}

fn get_end_point(request: &str) -> &str {
    let s: Vec<&str> = request.split_whitespace().collect();
    s[1]
}

fn get_protocol(request: &str) -> &str {
    let s: Vec<&str> = request.split_whitespace().collect();
    s[0]
}

fn get_page<'a>(routes: &HashMap<String, EndPoint>, end_point: &'a str, static_folders: &Vec<String>) -> String {
    let result = routes.get(end_point);

    match result {
        Some(response) => {
            pre_render(response)
        },
        None => {
            let static_file = find_static_file(end_point, static_folders);

            match static_file {
                Some(response) => {
                    response
                },
                None => {
                    let page404 = routes.get("404");
                    match page404 {
                        Some(response) => {
                            pre_render(response)
                        },
                        None => {
                            "404 page not found".to_string()
                        }
                    }
                }
            }
        }
    }
}

fn find_static_file(path: &str, static_folders: &Vec<String>) -> Option<String> {
    for folder in static_folders.iter() {
        let file = fs::read_to_string(format!("{}{}", folder, path));

        match file {
            Ok(file) => {
                return Some(file);
            },
            Err(_) => {
                continue;
            }
        }
    }
    None
}

fn register_end_point(routes: &mut HashMap<String, EndPoint>, end_point: &str, file_path: &str, f: fn(&str) -> String) {
    let html_code = fs::read_to_string(file_path).expect("File not found");

    let page_info = EndPoint {
        page: html_code.to_owned(),
        renderer: f
    };

    routes.insert(end_point.to_owned(), page_info);
}

fn pre_render(response: &EndPoint) -> String  {
    let mut response_to = response.page.clone();

    // Ifs
    let to_change: Vec<_> = response.page.match_indices("{pap:").collect();
    for (_, _) in to_change {
        let start_tag_index = response_to.find("{pap:").unwrap();


        let mut closing_tag_index = start_tag_index + 1;
        let mut j = 0;
        for i in response_to[start_tag_index..].chars() {
            if i == '{' {
                j += 1;
            }

            if i == '}' {
                j -= 1;

                if j == 0 {
                    break;
                }
            }
            closing_tag_index += 1;
        }

        println!("{}", response_to[start_tag_index..closing_tag_index].to_string());
        


    }

    // Render Variables
    let to_change: Vec<_> = response.page.match_indices("{pap=").collect();


    for (_, _) in to_change {
        let start_tag_index = response_to.find("{pap=").unwrap();
        
        let closing_tag_index = response_to[start_tag_index..].to_string().find("}").expect(format!("no closing tag found").as_str());

        let instruction = &response_to[start_tag_index + 6 .. start_tag_index + closing_tag_index];

        let evaluated = (response.renderer)(instruction);
        response_to.replace_range(start_tag_index .. start_tag_index + closing_tag_index + 1, evaluated.as_str());

    }
    
    response_to
}

fn render<T>(instruction: &str, objects: Option<Vec<Template<T>>>) -> String where T: Serialize {

    match objects {
        Some(objects) => {
            render_with_objects(instruction, objects)
        },
        None => {
            render_without_objects(instruction)
        }
    }


}

fn render_with_objects<T>(instruction: &str, objects: Vec<Template<T>>) -> String where T: Serialize {
    let mut t = Expr::new(instruction);

    for object in objects.iter() {

        let t2 = t.clone().value(object.name, &object.value);
        t = t2;
    }

    let result = t.exec();

    match result {
        Ok(v) => {
            v.to_string()
        },
        Err(e) => e.to_string(),
    }
}

fn render_without_objects(instruction: &str) -> String {
    let t = Expr::new(instruction);
    let result = t.exec();


    match result {
        Ok(v) => {
            v.to_string()
        },
        Err(e) => e.to_string(),
    }
}

fn register_static_folder(folder_path: &str, static_folders: &mut Vec<String>) {
    static_folders.push(folder_path.to_string().to_lowercase());
}