use std::{collections::HashMap, net::TcpStream};

use exp::{send_html, send_json, get_html_page, render, EndPoint, Protocols};

mod exp;

fn main() {
    let mut get_routes: HashMap<String, EndPoint> = HashMap::new();
    let mut post_routes: HashMap<String, EndPoint> = HashMap::new();

    let mut static_folders: Vec<String> = Vec::new();

    exp::register_static_folder("html/static", &mut static_folders);

    exp::register_end_point(&mut get_routes, "*", Protocols::GET, |_req: &Vec<String>, stream: TcpStream| {
        let page = get_html_page("html/404.ftml").unwrap();
        send_html(stream, page);
    });

    exp::register_end_point(&mut get_routes, "/", Protocols::GET, |_req: &Vec<String>, stream: TcpStream| {
        let page = get_html_page("html/index.html").unwrap();

        let mut object = HashMap::new();
        object.insert("foos", vec!["Hello", "world", "!"]);

        let temp = exp::Template {
            name: "obj",
            value: object,
        };
        let temp2 = vec![temp];

        let page = render(page, Some(temp2));
        
        // let page: String = render::<String>(page, None);
        send_html(stream, page);
    });

    exp::register_end_point(&mut get_routes, "/json", Protocols::GET, |_req: &Vec<String>, stream: TcpStream| {
        send_json(stream, "{
            \"status\": \"success\",
            \"message\": \"Welcome to the home page\"
        }".to_string());
    });
    
    exp::register_end_point(&mut post_routes, "/test", Protocols::POST, |_req: &Vec<String>, _stream: TcpStream| {
        println!("{}", _req[0]);
    });

    let mut routes: HashMap<Protocols, &HashMap<String, EndPoint>> = HashMap::new();
    routes.insert(Protocols::GET, &get_routes);
    routes.insert(Protocols::POST, &post_routes);

    exp::start_server("127.0.0.1:7878", routes, static_folders);
}