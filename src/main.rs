use chrono::{DateTime, FixedOffset};
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::time::Duration;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom, BufRead};
use std::env;
use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let watch_path = &args[1];
    let hostname = hostname::get().unwrap().into_string().unwrap();
    let cri_log_lines_regex = Regex::new(r"^([0-9-T:Z.]*) (std(?:err|out)) ([\S:]+) (.*)$").unwrap();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(watch_path, RecursiveMode::Recursive).unwrap();

    println!("Watching path: {}", watch_path);

    let mut watched_files: HashMap<String, u64> = HashMap::new();

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::NoticeWrite(path) => {
                        let path = path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("NoticeWrite: {}", path);
                    }
                    DebouncedEvent::NoticeRemove(path) => {
                        let path = path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("NoticeRemove: {}", path);
                    }
                    DebouncedEvent::Create(path) => {
                        let path = path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("Create: {}", path);
                    }
                    DebouncedEvent::Write(path) => {
                        let path = path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("Write: {}", path.clone());

                        let pos = watched_files.entry(path.clone()).or_insert(0);

                        let f = File::open(&path).unwrap();
                        let mut reader = BufReader::new(f);

                        reader.seek(SeekFrom::Start(pos.clone() as u64)).unwrap();

                        loop {
                            let mut line = String::new();
                            let result = reader.read_line(&mut line);

                            match result {
                                Ok(len) => {
                                    if len > 0 {
                                        *pos += len as u64;
                                        println!("File: {}", path);
                                        let log_lines = parse_cri_log_lines(
                                            &cri_log_lines_regex,
                                            line
                                        );
                                        println!("Parsed Log Lines: {:?}", log_lines);
                                    } else {
                                        break;
                                    }
                                }
                                Err(err) => {
                                    println!("Some err: {}", err);
                                }
                            }
                        }
                    }
                    DebouncedEvent::Chmod(path) => {
                        let path = path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("Chmod: {}", path);
                    }
                    DebouncedEvent::Remove(path) => {
                        let path = path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("Remove: {}", path);
                    }
                    DebouncedEvent::Rename(source_path, dest_path) => {
                        let path = source_path.into_os_string().into_string().unwrap();

                        if path.contains(&hostname) {
                            continue;
                        }

                        println!("Rename from {} to {}", path, dest_path.into_os_string().into_string().unwrap());
                    }
                    DebouncedEvent::Rescan => {
                        println!("Rescan");
                    }
                    DebouncedEvent::Error(err, path) => {
                        match path {
                            Some(path) => {
                                let path = path.into_os_string().into_string().unwrap();

                                if path.contains(&hostname) {
                                    continue;
                                }

                                println!("Error {} path {}", err, path);
                            }
                            None => {
                                println!("Error {}", err);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        };
    }
}

#[derive(Debug, PartialEq)]
enum OutputStream {
    StdOut,
    StdErr,
}

#[derive(Debug, PartialEq)]
struct LogLine {
    datetime: DateTime<FixedOffset>,
    stream: OutputStream,
    tags: Vec<String>,
    line: String,
}

fn parse_cri_log_lines(cri_log_lines_regex: &Regex, lines: String) -> Vec<LogLine> {
    let mut log_lines = Vec::<LogLine>::new();
    for line in lines.split("\n") {
        if let Some(log_line) = parse_cri_log_line(cri_log_lines_regex, line.to_string()) {
            log_lines.push(log_line);
        }
    }

    log_lines
}

fn parse_cri_log_line(cri_log_lines_regex: &Regex, line: String) -> Option<LogLine> {
    match cri_log_lines_regex.captures(&line) {
        Some(cap) => {
            Some(LogLine {
                datetime: DateTime::parse_from_rfc3339(&cap[1]).unwrap(),
                stream: if &cap[2] == "stdout" { OutputStream::StdOut } else { OutputStream::StdErr },
                tags: cap[3].split(':').map(|i| i.to_string()).collect(),
                line: cap[4].to_string(),
            })
        }
        None => {
            println!("Found line that didn't match regex: \"{}\"", line);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_rfc3339() {
        let dt = DateTime::parse_from_rfc3339("2021-01-16T02:02:11.040118643Z").unwrap();
        assert_eq!(dt.timestamp_nanos(), 1610762531040118643);
        let dt2 = DateTime::parse_from_rfc3339("2021-01-16T10:26:39.765975201Z").unwrap();
        assert_eq!(dt2.timestamp_nanos(), 1610792799765975201);
        let dt3 = DateTime::parse_from_rfc3339("2021-01-16T07:21:59.755423898Z").unwrap();
        assert_eq!(dt3.timestamp_nanos(), 1610781719755423898);
        let dt4 = DateTime::parse_from_rfc3339("2021-01-11T17:23:27.333350133Z").unwrap();
        assert_eq!(dt4.timestamp_nanos(), 1610385807333350133);
    }

    #[test]
    fn test_parsing_cri_log_line() {
        let test_log = "2021-01-11T17:23:43.253214031Z stderr F I0111 17:23:43.252724       1 serving.go:312] Generated self-signed cert (apiserver.local.config/certificates/apiserver.crt, apiserver.local.config/certificates/apiserver.key)";

        let log_lines = parse_cri_log_lines(test_log.to_string());
        assert_eq!(log_lines, vec![
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-11T17:23:43.253214031Z").unwrap(),
                stream: OutputStream::StdErr,
                tags: vec!["F".to_string()],
                line: "I0111 17:23:43.252724       1 serving.go:312] Generated self-signed cert (apiserver.local.config/certificates/apiserver.crt, apiserver.local.config/certificates/apiserver.key)".to_string(),
            }
        ]);

        let test_log2 = "2021-01-15T17:52:00.032546259Z stdout F Response {
2021-01-15T17:52:00.032616721Z stdout F   \"status\": 200,
2021-01-15T17:52:00.032635758Z stdout F   \"headers\": {";

        let log_lines = parse_cri_log_lines(test_log2.to_string());
        assert_eq!(log_lines, vec![
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-15T17:52:00.032546259+00:00").unwrap(),
                stream: OutputStream::StdOut,
                tags: vec!["F".to_string()],
                line: "Response {".to_string(),
            },
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-15T17:52:00.032616721+00:00").unwrap(),
                stream: OutputStream::StdOut,
                tags: vec!["F".to_string()],
                line: "  \"status\": 200,".to_string(),
            },
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-15T17:52:00.032635758+00:00").unwrap(),
                stream: OutputStream::StdOut,
                tags: vec!["F".to_string()],
                line: "  \"headers\": {".to_string(),
            }
        ]);

        let test_log3 = "2021-01-15T17:52:00.049462502Z stdout F ^[[0mPOST /gaction/fulfillment ^[[32m200^[[0m 10.169 ms - 161^[[0m
2021-01-16T10:11:07.88429767Z stdout F ^[[0mGET / ^[[33m404^[[0m 4.801 ms - 139^[[0m
2021-01-17T17:15:19.906752568Z stdout F ^[[0mGET / ^[[33m404^[[0m 4.442 ms - 139^[[0m";

        let log_lines = parse_cri_log_lines(test_log3.to_string());
        assert_eq!(log_lines, vec![
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-15T17:52:00.049462502Z").unwrap(),
                stream: OutputStream::StdOut,
                tags: vec!["F".to_string()],
                line: "^[[0mPOST /gaction/fulfillment ^[[32m200^[[0m 10.169 ms - 161^[[0m".to_string(),
            },
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-16T10:11:07.88429767Z").unwrap(),
                stream: OutputStream::StdOut,
                tags: vec!["F".to_string()],
                line: "^[[0mGET / ^[[33m404^[[0m 4.801 ms - 139^[[0m".to_string(),
            },
            LogLine {
                datetime: DateTime::parse_from_rfc3339("2021-01-17T17:15:19.906752568Z").unwrap(),
                stream: OutputStream::StdOut,
                tags: vec!["F".to_string()],
                line: "^[[0mGET / ^[[33m404^[[0m 4.442 ms - 139^[[0m".to_string(),
            },
        ]);
    }
}
