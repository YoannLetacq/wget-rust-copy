use url::Url;
use reqwest::*;
use reqwest::header::CONTENT_LENGTH;
use reqwest::header::CONTENT_DISPOSITION;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use chrono::{Utc, DateTime};
use regex::Regex;
use structopt::StructOpt;



#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "Mon Wget", about = "Download manager")]
struct Opt {
    #[structopt(short ="B", help = "Sets the URL to download")]
    url: String,

    #[structopt(short = "O", long, help = "Sets the output filename")]
    output: Option<String>,

    #[structopt(short = "P", long, help = "Sets the output directory")]
    directory: Option<PathBuf>,

    #[structopt(long, help = "Sets the download speed limit")]
    rate_limit: Option<String>,

    #[structopt(short = "i", long, help = "Sets the input file containing URLs to download")]
    input_file: Option<PathBuf>,

    #[structopt(long, help = "Mirror a website")]
    mirror: bool,

    #[structopt(short="R", long, use_delimiter = true, parse(try_from_str = parse_comma_separated), help = "Sets the file suffixes to reject when mirroring a website")]
    reject: Option<Vec<String>>,

    #[structopt(short = "X", long = "exclude", use_delimiter = true, parse(try_from_str = parse_comma_separated), help = "Sets the directories to exclude when mirroring a website")]
    exclude: Option<Vec<String>>,
}

fn parse_comma_separated(s: &str) -> std::result::Result<String, &'static str> {
    Ok(s.to_string())
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    if let Some(input_file) = opt.input_file.clone() {
        process_input_file(opt, input_file).await;
    } else if opt.mirror {
        mirror_website(opt).await;
    } else {
        download_file(opt).await;
    }
}

async fn download_file(opt: Opt) {
    let url = opt.url;
    let output_filename = opt.output;
    let destination_directory = opt.directory;
    let rate_limit = opt.rate_limit;

    println!("Téléchargement de {}", url);

    let response = reqwest::get(&url).await.expect("Échec de la requête");
    let headers = response.headers();

    // Display response status
    let status = response.status();
    println!("Status: {}", status);

    // Display content length
    if let Some(content_length) = headers.get(CONTENT_LENGTH) {
        let content_length_bytes = content_length.to_str().unwrap_or("Unknown");
        let content_length_human = format_size(content_length_bytes.parse::<u64>().unwrap_or(0));
        println!("Content Length: {} ({})", content_length_bytes, content_length_human);
    }

    let path = match output_filename {
        Some(filename) => destination_directory.unwrap_or_else(|| PathBuf::from(".")).join(filename),
        None => {
            println!("Getting Content-Disposition header...");
if let Some(content_disposition) = headers.get(CONTENT_DISPOSITION) {
    let content_disposition = content_disposition.to_str().expect("Échec de la conversion en chaîne");
    let re = Regex::new(r#"O="(?P<filename>.+)""#).expect("Échec de la création de l'expression régulière");
    println!("Applying regex...");
    if let Some(caps) = re.captures(content_disposition) {
        println!("Joining path...");
        destination_directory
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&caps["filename"])
    } else {
        let default_filename = url.split('/').last().expect("Impossible d'extraire le nom de fichier de l'URL");
        destination_directory
            .unwrap_or_else(|| PathBuf::from("."))
            .join(default_filename)
    }
} else {
    let default_filename = url.split('/').last().expect("Impossible d'extraire le nom de fichier de l'URL");
    destination_directory
        .unwrap_or_else(|| PathBuf::from("."))
        .join(default_filename)
}

        }
    };

    let mut file = File::create(&path).expect("Échec de la création du fichier");
    let bytes = response.bytes().await.expect("Échec de la lecture du flux de bytes");
    
    let mut pos = 0;
    let length = bytes.len();
    let mut limit: Option<u32> = None;
    
    if let Some(rate_limit) = rate_limit {
        let rate_limit = rate_limit.trim().to_lowercase();
    
        if rate_limit.ends_with("k") {
            if let Ok(speed_limit) = rate_limit[..rate_limit.len() - 1].parse::<u32>() {
                limit = Some(speed_limit * 1024);
            }
        } else if rate_limit.ends_with("m") {
            if let Ok(speed_limit) = rate_limit[..rate_limit.len() - 1].parse::<u32>() {
                limit = Some(speed_limit * 1024 * 1024);
            }
        } else {
            if let Ok(speed_limit) = rate_limit.parse::<u32>() {
                limit = Some(speed_limit);
            }
        }
    }
    
    let progress = ProgressBar::new(length as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.>} [{elapsed_precise}] [{bar:40.green}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-"),
    );

    let start_time: DateTime<Utc> = Utc::now();

    while pos < length {
        let chunk_size = if let Some(speed_limit) = limit {
            std::cmp::min(speed_limit as usize, length - pos)
        } else {
            length - pos
        };

        let bytes_written = file.write(&bytes[pos..pos + chunk_size]).expect("Échec de l'écriture dans le fichier");
        pos += bytes_written;
        progress.inc(bytes_written as u64);

        if let Some(speed_limit) = limit {
            let sleep_duration = Duration::from_secs_f32(chunk_size as f32 / speed_limit as f32);
            tokio::time::sleep(sleep_duration).await;
        }
    }

    progress.finish();

    let end_time: DateTime<Utc> = Utc::now();

    println!("Téléchargement terminé : {}", url);
    println!("Fichier enregistré sous : {}", path.display());
    println!("Start time: {}", start_time.format("%Y-%m-%d %H:%M:%S"));
    println!("End time: {}", end_time.format("%Y-%m-%d %H:%M:%S"));
}


async fn process_input_file(opt: Opt, input_file: PathBuf) {
    let file = File::open(input_file).expect("Échec de l'ouverture du fichier d'entrée");
    let reader = io::BufReader::new(file);
    let urls: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();
    let mut tasks = vec![];

    for url in urls {
        let opt = opt.clone();
        let task = tokio::spawn(async move {
            let mut opt_with_url = opt;
            opt_with_url.url = url;
            download_file(opt_with_url).await;
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("Échec de la jointure du thread");
    }
}


async fn mirror_website(opt: Opt) {
    let url = opt.url;
    let output_directory = opt.directory;
    let reject_list = opt.reject;
    let exclude_list = opt.exclude;
    let base_url = Url::parse(&url).expect("Échec de l'analyse de l'URL de base");
    let client = Client::new();
    let response = client.get(&url).send().await.expect("Échec de la requête");
    let html = response.text().await.expect("Échec de la lecture de la réponse HTML");
    let document = scraper::Html::parse_document(&html);
    let mut visited_urls: Vec<String> = Vec::new();
    let selector = scraper::Selector::parse("a, link, img").expect("Échec de l'analyse du sélecteur");

    for element in document.select(&selector) {
        if let Some(attr) = element.value().attr("href").or_else(|| element.value().attr("src")) {
            let cleaned_attr = attr.replace("//", "/"); // Supprimez les doubles /
            let mut url = base_url.clone();
            url.set_path(&cleaned_attr);

            if !visited_urls.contains(&url.to_string()) {
                visited_urls.push(url.to_string());
                println!("URL visitée : {}", url);
            }
        }
    }

    let mut tasks = vec![];

    for visited_url in visited_urls {
        if let Some(reject_list) = &reject_list {
            let should_reject = reject_list.iter().any(|suffix| visited_url.ends_with(&suffix.to_lowercase()));
            if should_reject {
                continue;
            }
        }
        if let Some(exclude_list) = &exclude_list {
            let should_exclude = exclude_list.iter().any(|path| {
                let url_path = Url::parse(&visited_url).ok().map(|u| u.path().to_lowercase());
                url_path.map(|p| p.starts_with(&path.to_lowercase())).unwrap_or(false)
            });
            if should_exclude {
                continue;
            }
        }

        let client = client.clone();
        let output_directory = output_directory.clone();

        let task = tokio::spawn(async move {
            let response = client.get(&visited_url).send().await.expect("Échec de la requête");
            let headers = response.headers();
            let mut path = des_path(&visited_url, &output_directory, &headers).expect("Échec de la création du chemin de destination");
            let is_directory = visited_url.ends_with("/");

            // Si l'URL est un répertoire, ajoutez un nom de fichier par défaut
            if is_directory {
                path = path.join("index.html");
            }

            let parent_path = path.parent().expect("Échec de la récupération du chemin parent");
            
            // Créez le répertoire parent s'il n'existe pas
            fs::create_dir_all(parent_path).expect("Échec de la création du répertoire");
            
            let  file = create_or_open_file(&path);
            let bytes = response.bytes().await.expect("Échec de la lecture du flux de bytes");
            file.expect("Echec de l'ouverture du fichier").write_all(&bytes).expect("Échec de l'écriture dans le fichier");
            println!("Téléchargé : {}", visited_url);
        });

        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("Échec de l'exécution de la tâche");
    }

    println!("Téléchargement terminé.");
}


fn create_or_open_file(path: &Path) -> io::Result<fs::File> {
    if path.exists() {
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other, 
                format!("{} is a directory, not a file", path.display())
            ));
        }
    }
    
    // Proceed to create or open the file
    fs::File::create(path)
}

fn des_path(url: &str, destination_directory: &Option<PathBuf>, headers: &reqwest::header::HeaderMap) -> std::result::Result<PathBuf, String> {
    let content_disposition = headers.get(CONTENT_DISPOSITION);
    let mut filename = if let Some(content_disposition) = content_disposition {
        let content_disposition = content_disposition.to_str().expect("Échec de la conversion en chaîne");
        let re = Regex::new(r#"O="(?P<filename>.+)""#).expect("Échec de la création de l'expression régulière");
        if let Some(caps) = re.captures(content_disposition) {
            caps["filename"].to_owned()
        } else {
            let default_filename = url.split('/').last().expect("Impossible d'extraire le nom de fichier de l'URL");
            if default_filename.is_empty() {
                "default".to_owned()
            } else {
                default_filename.to_owned()
            }
        }
    } else {
        let default_filename = url.split('/').last().expect("Impossible d'extraire le nom de fichier de l'URL");
        if default_filename.is_empty() {
            "default".to_owned()
        } else {
            default_filename.to_owned()
        }
    };
    let destination_directory = Path::new(destination_directory.as_deref().unwrap_or_else(|| Path::new(".")));
    let mut path = Path::new(destination_directory).join(&filename);

    // Check if a directory of the same name exists. If it does, modify the filename
    if path.is_dir() {
        filename = format!("{}_file", filename);
        path = Path::new(destination_directory).join(&filename);
    }

    Ok(path)
}



fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}