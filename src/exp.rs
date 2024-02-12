#![allow(unused_variables)]
#![warn(dead_code)]

use std::{
    collections::HashMap, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};

use eval::Expr;
use serde::Serialize;

pub struct EndPoint {
    renderer: fn(req: &Vec<String>, stream: TcpStream),
}

pub struct Template<T> where T: Serialize {
    pub name: &'static str,
    pub value: T,
} 


pub fn start_server(addr: &str, routes: HashMap<String, EndPoint>, static_folders: Vec<String>) {

    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &routes, &static_folders);
    }
}

fn handle_connection(mut stream: TcpStream, routes: &HashMap<String, EndPoint>, static_folders: &Vec<String>) {
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
        "POST" => println!("Request: {:#?}", http_request[0]),
        _ => return println!("Unknown request"),
    }
    let end_point = get_end_point(&http_request[0]);
    let page = get_page(&routes, end_point, &static_folders);
    match page {
        Some(page) => {
            (page.renderer)(&http_request, stream);
        },
        None => {
            let static_file = find_static_file(end_point, static_folders);

            match static_file {
                Some(response) => {
                    send_html(stream, response);
                },
                None => {
                    let page404 = routes.get("*");
                    if let Some(page404) = page404 {
                        (page404.renderer)(&http_request, stream);
                    }
                }
            }
        }
    }
}

pub fn send_html(mut stream: TcpStream, file: String) {
    let status_line = "HTTP/1.1 200 OK";
    let length = file.len();

    let response = format!(
        "{status_line}\r\nServer: FredWork/0.1.0 \n Content-Type: text/html \nContent-Length: {length}\r\n\r\n{file}",
    );

    stream.write_all(response.as_bytes()).unwrap();
}

pub fn send_json(mut stream: TcpStream, file: String) {
    let status_line = "HTTP/1.1 200 OK";
    let length = file.len();

    let response = format!(
        "{status_line}\r\nServer: FredWork/0.1.0\r\nContent-Type: application/json \r\nDataType: json \r\nContent-Length: {length}\r\n\r\n{file}",
    );

    stream.write_all(response.as_bytes()).unwrap();
}

pub fn get_html_page(path: &str) -> Option<String> {
    let file = fs::read_to_string(path);
    match file {
        Ok(file) => {
            Some(file.to_string())
        },
        Err(_) => {
            None
        }
    }
}

fn get_end_point(request: &str) -> &str {
    let s: Vec<&str> = request.split_whitespace().collect();
    s[1]
}

fn get_protocol(request: &str) -> &str {
    let s: Vec<&str> = request.split_whitespace().collect();
    s[0]
}

fn get_page<'a>(routes: &'a HashMap<String, EndPoint>, end_point: &'a str, static_folders: &Vec<String>) -> Option<&'a EndPoint> {
    routes.get(end_point)
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

pub fn register_end_point(routes: &mut HashMap<String, EndPoint>, end_point: &str, f: fn(req: &Vec<String>, stream: TcpStream)) {
    let page_info = EndPoint {
        renderer: f
    };

    routes.insert(end_point.to_owned(), page_info);
}

pub fn register_static_folder(folder_path: &str, static_folders: &mut Vec<String>) {
    static_folders.push(folder_path.to_string().to_lowercase());
}

pub fn render<T>(page: String, objects: Option<Vec<Template<T>>>) -> String where T: Serialize {

    // Render Variables

        
    let to_change: Vec<_> = page.match_indices("<@=").collect();
    let mut modified_page = page.clone();

    for (_, _) in to_change {
        let start_tag_index = modified_page.find("<@=").unwrap();
        
        let closing_tag_index = modified_page[start_tag_index..].to_string().find(">").expect(format!("no closing tag found").as_str());

        let instruction = &modified_page[start_tag_index + 3 .. start_tag_index + closing_tag_index];

        let evaluated = render_with_objects(instruction, &objects);
        modified_page.replace_range(start_tag_index .. start_tag_index + closing_tag_index + 1, evaluated.as_str());

    }
    modified_page


}

fn render_with_objects<T>(instruction: &str, objects: &Option<Vec<Template<T>>>) -> String where T: Serialize {
    let mut t = Expr::new(instruction);

    if let Some(objects) = objects {
        for object in objects.iter() {
            let t2 = t.clone().value(object.name, &object.value);
            t = t2;
        }
    }

    let result = t.exec();

    match result {
        Ok(v) => {
            v.to_string()
        },
        Err(e) => e.to_string(),
    }
}