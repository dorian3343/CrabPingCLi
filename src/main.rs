
use std::env;
use chrono::{Utc, DateTime, Duration};
use hyper::{Client, body::to_bytes, http::Uri as HyperUri};
use hyper_tls::HttpsConnector;
use std::sync::mpsc;

#[derive(PartialEq)]
#[derive(Debug)]

/*
ReqObj is a collection of data about the request:
-benchmark: the time of the request in milliseconds
-id: the id of the request
-status: the http  status that the request returned
-contents: what ever the body of the request returned
*/

struct ReqObj {
    benchmark: i64,
    id: u32,
    status: String,
    contents: String,
}

impl Default for ReqObj {
    fn default() -> Self{
        ReqObj {
            benchmark: 0,
            id: 0,
            status: String::default(),
            contents: String::default(),
        }
    }
}

async fn send_req(adr: &str, id: &u32) -> Result<ReqObj, Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = match adr.parse::<HyperUri>(){
            Ok(val) => val,
            Err(_err) => {panic!("Failed to parse url address")},
    };
    let start: DateTime<Utc> = Utc::now();
    let res = client.get(uri.clone()).await?;
    let end: DateTime<Utc> = Utc::now();
    let dur: Duration = end - start;

    let status = res.status().clone();
    let body_bytes = to_bytes(res.into_body()).await?;
    let body_str = String::from_utf8(body_bytes.to_vec())?; 

   let obj = ReqObj {
        benchmark: dur.num_milliseconds(),
        id: *id,
        status: status.to_string(),
        contents: body_str,
    };

    Ok(obj)
}


//benchmark caluclation
fn calculate_average(vec: &Vec<i64>) -> f64 {
    let sum: i64 = vec.iter().sum();
    let count = vec.len() as f64;
    let average = sum as f64 / count;
    average
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = env::args().collect();
    match args.len() {
        1 => {
            // Display help menu
        println!("Welcome to CrabPing! An endpoint tester made in Rust!");
        println!("=====================================================");
        println!("Usage: CrabPing [HttpReq] [Amount]");
        println!("------------------------------------");
        println!("Running just 'CrabPing' shows this help menu.");
        println!("[HttpReq]: The endpoint you want to hit.");
        println!("[Amount]: How many requests you want to send (max: 200).");
        println!("=====================================================");
        println!("Source Code: https://github.com/dorian3343/CrabPing");
        println!("Made by: Dorian Kalaczynski");
        println!("=====================================================");
        },
        2 => {
            //if only one request
            let request = match send_req(&args[1].to_string(),&0).await {
                Ok(val) => val,
                Err(_) => {
                //this part needs a revamp, however it basically creates a default object
                //and matches it to check for errors
                    ReqObj::default()
                },
            };

            if request == ReqObj::default(){
                println!("Error! URL parse failed")
            }else{
            println!("Id: [{}]\nStatus: [{}]\nContents: [{}]\nBenchmark: [{} ms]",request.id,
                     request.status,
                     request.contents,
                     request.benchmark,
                     );
            }
        },
        3 => { if args[2].parse::<u32>().is_ok(){
                if args[2].parse::<u32>().unwrap() < 1 {
                    println!("Error! Minimum requests is 1");
                }else if args[2].parse::<u32>().unwrap() > 200 {
                    println!("Error! Maximum requests is 200");
                }else{
                    let (tx,rx) = mpsc::channel();
                    let mut handles = vec![];
                    for i in 0.. args[2].parse::<u32>().unwrap(){
                        let tx_clone = tx.clone();
                        let adr = args[1].clone();
                        //create a new thread for every request to parralelize the requests
                        let handle = tokio::spawn(async move {
                            let adr_clone = adr.clone();
                            let request = send_req(&adr_clone,&i).await.unwrap();
                            tx_clone.send(request).expect("Failed to send result");
                        });
                        handles.push(handle);
                    }
                    let mut times = vec![];
                    //wait for the reqs end
                    for _ in handles {
                        let catch = rx.recv().expect("Failed to send result");
                     println!("Id: [{}]\nStatus: [{}]\nContents: [{}]\nBenchmark:[{} ms]",
                              catch.id,
                              catch.status,
                              catch.contents,
                              catch.benchmark);
                     times.push(catch.benchmark);
                    }
                    let highest = *times.iter().max().unwrap();
                    let smallest = *times.iter().min().unwrap();
                    let avg = calculate_average(&times);
                    println!("Benchmark Results:\n===============\nFastest:[{} ms]\nSlowest:[{} ms]\nAverage:[{} ms]"
                             ,smallest,highest,avg);
                }
            }else{
                println!("Error! Non numeric third paramater")
            }
    },
        _ => {},
    };
}

