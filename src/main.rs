use std::f32::consts::E;
use std::{str,env, result};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Error, Write};
use std::sync::Arc;
use std::process::Command;
use base64::{encode, decode};
use axum::{
    response::Html, 
    routing::get, 
    Router,
    extract::{Query}, 
    Extension,
    http::{request::Parts, StatusCode}
};
use walkdir::WalkDir;
use serde::{de, Deserialize, Deserializer};
use tracing::{info,debug,error,Level};
use tracing_subscriber::fmt::format;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use clap::{Parser, Arg};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "~/")]
    root: String,
}


struct State {
    args:Args
}


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_env("AXUM_CONVERTER_LOG"))
    .init();

    if env::var("AXUM_CONVERTER_LOG").is_err() {
        env::set_var("AXUM_CONVERTER_LOG", "axum_converter=info");
    }

    let args = Args::parse();
    let state = Arc::new(State { args});

    let app =  Router::new().route("/", get(handler)).layer(Extension(state));

    info!("https://0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(Query(params): Query<Params>,state: Extension<Arc<State>>) -> String {
    debug!("{}", state.args.root);
    let target=params.target.unwrap();
    match params.url{
        Some(url)=>{
            match fetch_node_from_url(target,url,&state.args.root).await {
                Ok(result)=>result,
                Err(e)=>"获取节点失败".to_string(),
            }
              
        },
        None=> {
            error!("订阅网址不能为空!");
            "订阅网址不能为空".to_string()
        },
    }
}


async fn fetch_node_from_url(target:String,url:String,root:&String)->Result<String, reqwest::Error>{
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    let fetch_path = Path::new(root).join("fetchnode");
    let targets: Vec<&str> = target.split(',').collect();
    let mut output = fs::File::create(fetch_path).unwrap();
    match  write!(output, "{}", &body){
        Ok(_)=>{
            debug!("远程节点写入成功!");
        },
        Err(e)=>{
            error!("远程节点写入出错!");
        }   
    }
    let output = Command::new("subconverter")
    .current_dir(root)
    .arg("-g")
    .arg("-f")
    .arg(format!("{}/generate.ini",root))
    .output()
    .expect("failed to execute process");

    let hello = output.stdout;
    let s = match String::from_utf8(hello) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    let nodes = Path::new(root).join("nodes");
    let result_path = Path::new(root).join("result.txt");
    let mut f = fs::File::create(&result_path).expect("Unable to create file");
    for entry in WalkDir::new(&nodes) {
        match  entry{
            Ok(entry)=>{
                debug!("read file:{}",entry.path().display());
                if !entry.path().to_str().unwrap().ends_with(".txt") {
                    continue;
                }
                let file_full_name = entry.file_name().to_str().unwrap();
                let filename = file_full_name.get(0..(file_full_name.len()-4)).unwrap();
                if !targets.contains(&filename) { 
                    continue;
                } 
                match  fs::read_to_string(entry.path()) {
                    Ok(result)=>{
                        match decode(result){
                            Ok(res)=>{
                                let node_list = String::from_utf8(res.clone()).unwrap();
                                if node_list.trim().is_empty() {
                                    error!("{}没有解析到结点!",entry.path().to_str().unwrap());
                                    continue;
                                }
                                f.write_all(&res).expect("节点写入失败");
                                //f.write_all(node_list.as_bytes()).expect("节点写入失败");
                            },
                            Err(e)=>{
                                error!("base64解码{}出错!",entry.path().to_str().unwrap());
                            }
                        }
                    },
                    Err(e)=>{
                        error!("读取文件出错!");
                    }   
                }
            },
            Err(e)=>{
                error!("找不到文件或目录请检查参数!");
            }
        }
    }
    
    fs::remove_dir_all(&nodes).unwrap();
    fs::create_dir(&nodes).unwrap();

    let nodes_result = fs::read_to_string(&result_path.to_str().unwrap());
    Ok(encode(nodes_result.unwrap()))
}





#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Params {
    target:Option<String>,
    url: Option<String>,
}
