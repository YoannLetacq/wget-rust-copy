# WGET (RUST VERSION)

A simple implementation of wget, the popular file download utility, written in Rust.


### Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Dependencies](#dependencies)
- [Example](#example)
- [License](#license)
- [Author](#author)

## Features

- Simple file downloads from a given URL
- Customizable download directory
- Customizable output filename
- Automatic filename detection from URL if no output filename is provided



## Installation

Ensure you have [Rust installed](https://www.rust-lang.org/tools/install).

Then, clone the repository and build the project:

```sh
git clone https://github.com/yourusername/wget-rust.git
cd wget-rust
cargo build --release
```

The built executable will be located in ./target/release/.

## dependencies

Be sure you get in your Cargo.toml file the following dependencies:
```sh
[dependencies]

reqwest = { version = "0.11", features = ["default"] }
tokio = { version = "1", features = ["full"] }
regex = "1.4.5"
indicatif = "0.16.2"
chrono = "0.4.19"
url = "2.2.2"
futures = "0.3.15"
structopt = "0.3"
scraper = "0.12"

```
* reqwest (version 0.11): This crate allows you to make HTTP requests in Rust. It provides a high-level HTTP client with various features and customization options.

* tokio (version 1): Tokio is an asynchronous runtime for Rust. It provides an asynchronous execution environment and tools for writing asynchronous code. The "full" feature enables all the components of Tokio.

* regex (version 1.4.5): This crate provides support for regular expressions in Rust. It allows you to create and match regular expressions against text, providing powerful pattern matching capabilities.

* indicatif (version 0.16.2): Indicatif is a crate that helps you create interactive progress bars and other status indicators in the terminal. It provides an easy way to display progress and status updates to users during long-running operations.

* chrono (version 0.4.19): Chrono is a crate for date and time handling in Rust. It provides various types and functions for working with dates, times, time zones, durations, and formatting.

* url (version 2.2.2): The url crate offers parsing, manipulation, and resolution functionality for URLs. It allows you to parse and work with URLs, including extracting components and building URLs programmatically.

* futures (version 0.3.15): The futures crate provides a foundation for asynchronous programming in Rust. It introduces types and traits for working with asynchronous computations, including futures, streams, and asynchronous I/O.

* structopt (version 0.3): Structopt is a crate for parsing command-line arguments by defining a struct with attributes. It simplifies the process of defining and parsing command-line arguments in Rust applications.

* scraper (version 0.12): crate is commonly used for web scraping tasks in Rust. It provides functionality and utilities for parsing and manipulating HTML or XML documents to extract specific data.


## Usage

After building, you can use the program with the following command:
```sh
./target/release/wget-rust -B <url> -O=<output> -P=<directory>
```
If you encounter permission denied problem try to use:
```sh 
chmod +x wget
``` 

Don't refer to the audit test because i doesn't look the placement of the file. (It base the bin syntax as golang one).
## Example

Demonstration in terminal link bellow:

![App Screenshot](https://cdn.discordapp.com/attachments/975481270756835329/1125794009407836291/Screenshot_from_2023-07-04_16-23-06.png)


Where:

<url> is the URL of the file you want to download
<output> (optional) is the name you want the downloaded file to have
<directory> (optional) is the directory you want the file to be downloaded to
If no output filename or download directory are provided, the file will be downloaded to the current directory and its name will be derived from the URL.

## Licence 

This project is open source and available under the MIT License.


## Author 

Yoann "yyoannle" Letacq group.
