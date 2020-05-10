use clap::{App, AppSettings, SubCommand};
use std::path::Path;
use anyhow::Context;
use std::sync::Mutex;
use std::sync::Arc;
use askama::Template;

/// expected filesystem structure:
/// 
/// data/
///     abjasb3.json
///         {
///             "commit": {
///                 "sha1": "abjasb3309jag03r".
///                 "title": "Add integration test",
///                 "author": "Abc Def",
///                 "time": "",
///                 "url": "https://..github.com..",  <- optional
///             },
///             "benches": {
///                 "topic_name": {"unit": "ms", "mean": 3123123123.132},
///                 "read_100_files": {"unit": "ms", ""mean": 321, "std": 1}  <- std is optional
///             }
///         }
///     jkai0bj.json
///     92tjab9.json
///     ....
/// out/
///     compile_time.png  <- by default, plot the last 100 commits
///     test1.png
///     test2.png

/// src_dir: data/
/// dst_dir: out/


#[derive(serde::Deserialize)]
struct Commit {
    commit: CommitInfo,
    benches: std::collections::HashMap<String, BenchSample>,
}

#[derive(serde::Deserialize)]
struct CommitInfo {
    sha1: String,
    title: String,
    author: String,
    time: chrono::DateTime<chrono::Utc>,  // rfc3339, e.g. 1981-02-22T09:00Z
    url: Option<String>,
}

#[derive(serde::Deserialize)]
struct BenchSample {
    unit: String,
    mean: f64,
    #[serde(default, rename="std")]
    sd: f64,
}

#[derive(serde::Deserialize)]
struct Bench {
    // f64::NAN for hole
    name: String,
    unit: String,
    means: Vec<f64>,
    sds: Vec<f64>,
}

impl Bench {
    fn new(name:String, unit: String) -> Bench {
        Bench{name, unit, means:Vec::new(), sds:Vec::new()}
    }
}

#[derive(askama::Template)]
#[template(path = "template.html")]
struct IndexTemplate<'a> {
    render_data: &'a RenderData,
}

struct RenderData {
    benches: std::collections::HashMap::<String, Bench>,
    xlabels: Vec<String>,
}


fn plot(src_dir: &Path, dst_dir: &Path) -> anyhow::Result<RenderData> {
    std::fs::create_dir_all(dst_dir).context("create dst dir")?;

    // read commits
    let paths = std::fs::read_dir(src_dir).context("list src dir")?;
    let mut commits = Vec::new();
    for res in paths {
        let p = res.context("decode src file path as utf8")?.path();
        let s = std::fs::read_to_string(&p)?;

        let c :Commit = match p.extension().map(|s| s.to_str().unwrap()) {
            Some("json") => serde_json::from_str(&s)?,
            Some("toml") => toml::from_str(&s)?,
            _ => {eprintln!("unexpected file in src_dir: {}", p.display()); continue;}
        };
        commits.push(c);
    }

    // sort commits by commit time
    commits.sort_by(|a,b| a.commit.time.cmp(&b.commit.time));

    let mut benches = std::collections::HashMap::<String, Bench>::new();
    let mut xlabels = Vec::new();

    // get a list of bench topics
    for c in &commits {
        xlabels.push(format!("{} ({})", c.commit.title, (&c.commit.sha1[..6]).clone()));
        for (name, sample) in &c.benches {
            let e = benches.entry(name.clone()).or_insert(Bench::new(name.clone(),sample.unit.clone()));
            anyhow::ensure!(e.unit == sample.unit, "bench '{}' has two different units '{}' and '{}'", name, e.unit, sample.unit);
        }
    }

    for c in &commits {
        for (name, b) in &mut benches {
            if c.benches.contains_key(name) {
                let sample = &c.benches[name];
                b.means.push(sample.mean);
                b.sds.push(sample.sd);
            } else {
                b.means.push(f64::NAN);
                b.sds.push(f64::NAN);
            }
        }
    }
    Ok(RenderData{benches, xlabels})
}

#[actix_web::get("/")]
async fn serve_index(data: actix_web::web::Data<Arc<Mutex<RenderData>>>) -> impl actix_web::Responder {
    let render_data = data.lock().unwrap();
    let s = IndexTemplate {render_data: &render_data}.render().unwrap();
    actix_web::HttpResponse::Ok().body(s)
}

fn serve(src_dir: &Path, dst_dir: &Path) -> anyhow::Result<()> {
    // actix_web::web::Data requires 'static
    let render_data: Arc<Mutex<RenderData>> = Arc::new(Mutex::new(plot(src_dir, dst_dir).context("plot failed")?));

    // static serve dst_dir
    println!("actix run");
    actix_rt::System::new("server").block_on(async move {
        actix_web::HttpServer::new(move || actix_web::App::new()
        .app_data(actix_web::web::Data::new(render_data.clone()))
        .service(serve_index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
    })?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let matches = App::new("cocoball").version("prealpha")
    .author("github.com/elbaro/cocoball")
    .about("plot continuous benchmarking")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommand(
        SubCommand::with_name("plot").about("create the png files for the last n commits (default n = 100)")
    )
    .subcommand(
        SubCommand::with_name("serve").about("run the webserver to serve the latest plots (monitor filesystem)")
    ).get_matches();

    let sub = matches.subcommand.unwrap();
    match sub.name.as_ref() {
        "generate" => {
            plot("data".as_ref(), "out".as_ref()).context("plot failed")?;
        }
        "serve" => {
            serve("data".as_ref(), "out".as_ref()).context("serve failed")?;
        }
        _ => unreachable!()
    }
    Ok(())
}
