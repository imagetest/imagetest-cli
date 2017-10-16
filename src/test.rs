use std::fs::File;
use std::io::{Read, BufReader};
use std::io::{self};
use std::fmt::{self, Formatter, Display};

use clap::{Arg, ArgMatches, App, SubCommand};
use futures::{future, Future, Stream};
use hyper::Client;
use hyper::client::Request;
use hyper::header::{ContentLength, Authorization, Bearer};
use hyper;
use serde_json::{self, Value as Json};
use tokio_core::reactor::{Core, Handle};
use toml::{from_str};

use config::{TestImage, ConfigV1};
use ::exit_with_error;

pub const NAME: &'static str = "test";

type BoxFuture<T, E> = Box<Future<Item = T, Error = E>>;

#[derive(Deserialize, Debug)]
pub struct TestResponse {
    pub success: bool,
}

#[derive(Debug)]
pub struct TestResult<T> {
    pub test: T,
    pub response: TestResponse,
}

impl<T> TestResult<T> {
    pub fn format_single(&self) -> SingleCharacterFormat<&Self> {
        SingleCharacterFormat {
            inner: self,
        }
    }
}

pub struct SingleCharacterFormat<T> {
    inner: T
}

impl<'a, T> Display for SingleCharacterFormat<&'a TestResult<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.inner.response.success {
            write!(f, "✓")
        } else {
            write!(f, "✗")
        }
    }
}

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Run tests")
        .arg(Arg::with_name("config file")
             .long("config")
             .default_value(".imagetest.toml")
        )
}

pub fn run(matches: &ArgMatches) {
    let path = matches.value_of("config file").unwrap();

    let contents = ::config::read_config_file(path);
    let config_res: Result<ConfigV1, _> = from_str(&contents);
    let config = match config_res {
        Err(_) => {
            exit_with_error("Could not parse config file into expected config format", false);
            return;
        },
        Ok(val) => val,
    };
    submit_tests(&config.test_image, config.clone());
}

fn submit_tests(tests: &[TestImage], config: ConfigV1) {
    let mut core = Core::new().unwrap();

    let mut test_futures = Vec::new();

    for test in tests {
        let data = read_image_file(&test.path);
        let ftr = upload_image(&core.handle(), config.clone(), data, test.clone());
        test_futures.push(ftr);
    }

    test_futures = test_futures
        .into_iter()
        .map(|result| {
            let printed = result.then(|res| {
                if let Ok(ref result) = res {
                    print!("{}", result.format_single());
                }
                res
            });
            Box::new(printed) as Box<Future<Item=_, Error=_>>
        })
        .collect();

    let all = future::join_all(test_futures)
        .and_then(|results| {
            print!("\n");

            for result in results {
                if !result.response.success {
                    ::std::process::exit(1);
                }
            }

            Ok(())
        });

    core.run(all).unwrap();
}

fn upload_image(
    handle: &Handle,
    config: ConfigV1,
    data: Vec<u8>,
    test: TestImage
) -> BoxFuture<TestResult<TestImage>, hyper::Error> {
    let mut request = Request::new(
        hyper::Method::Post,
        format!("{}/beta/tests/submit?test_id={}", config.api_url(), test.test_id).parse().unwrap()
    );
    request.headers_mut().set(ContentLength(data.len() as u64));
    request.headers_mut().set(Authorization(Bearer { token: config.test_token() }));
    request.set_body(data);

    return Box::new(request_json(&handle, request)
        .and_then(|res| Ok(::serde_json::from_value(res).unwrap()))
        .and_then(|resp| {
            let result = TestResult {
                test,
                response: resp,
            };
            Ok(result)
        }));
}

fn request_json(handle: &Handle, request: Request) -> BoxFuture<Json, hyper::Error> {
    let res = Client::new(handle)
        .request(request)
        .and_then(|res| {
            res.body().concat2()
                .and_then(|body| {
                    Ok(serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            e
                        )
                    })?)
                })
        });

    return Box::new(res);
}

pub fn read_image_file(path: &str) -> Vec<u8> {
    let input = File::open(path).expect("Could not find file");
    let mut buffered = BufReader::new(input);

    let mut contents = Vec::new();
    buffered.read_to_end(&mut contents).expect("Could not read file");

    contents
}
