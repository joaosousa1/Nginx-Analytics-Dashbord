use sqlx::SqlitePool;
use std::{fs, os::unix::fs::MetadataExt};

// Convert Nginx log timestamp to ISO format
pub fn parse_nginx_timestamp_to_iso(raw_ts: &str) -> String {
    let clean_ts = raw_ts.split_whitespace().next().unwrap_or(raw_ts);
    let parts: Vec<&str> = clean_ts.split(':').collect();
    if parts.is_empty() {
        return raw_ts.to_string();
    }

    let date_parts: Vec<&str> = parts[0].split('/').collect();
    if date_parts.len() != 3 {
        return raw_ts.to_string();
    }

    let day = date_parts[0];
    let month_str = date_parts[1];
    let year = date_parts[2];

    let month_num = match month_str {
        "Jan" => "01", "Feb" => "02", "Mar" => "03", "Apr" => "04",
        "May" => "05", "Jun" => "06", "Jul" => "07", "Aug" => "08",
        "Sep" => "09", "Oct" => "10", "Nov" => "11", "Dec" => "12",
        _ => "01",
    };

    let time_part = if parts.len() >= 4 {
        format!("{}:{}:{}", parts[1], parts[2], parts[3])
    } else {
        "00:00:00".to_string()
    };

    format!("{}-{}-{} {}", year, month_num, day, time_part)
}

// Parse Nginx access logs and store them into SQLite with inode/rotation checks
pub async fn parse_and_store_logs(pool: &SqlitePool, domain: &str, log_path: &str) {
    let metadata = match fs::metadata(log_path) {
        Ok(m) => m,
        Err(_) => return,
    };
    
    let file_size = metadata.len();
    let current_inode = metadata.ino();

    let last_state: Option<(u64, i64)> = sqlx::query_as(
        "SELECT inode, last_position FROM log_states WHERE path = ?"
    )
    .bind(log_path)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let mut start_pos: u64 = 0;

    if let Some((saved_inode, saved_pos)) = last_state {
        if saved_inode == current_inode {
            let pos = saved_pos as u64;
            if file_size >= pos {
                start_pos = pos;
            } else {
                start_pos = 0;
            }
        } else {
            start_pos = 0;
        }
    }

    if file_size == start_pos && last_state.is_some() {
        return;
    }

    let mut file = match fs::File::open(log_path) {
        Ok(f) => f,
        Err(_) => return,
    };

    use std::io::{Read, Seek, SeekFrom};
    if file.seek(SeekFrom::Start(start_pos)).is_err() {
        return;
    }

    let mut new_content = String::new();
    if file.read_to_string(&mut new_content).is_err() {
        return;
    }

    for line in new_content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let mut parts = line.splitn(2, " - - [");
        let Some(ip) = parts.next() else { continue; };
        let Some(rest) = parts.next() else { continue; };

        let mut rest_parts = rest.splitn(2, "] \"");
        let Some(timestamp) = rest_parts.next() else { continue; };
        let Some(rest2) = rest_parts.next() else { continue; };

        let mut req_parts = rest2.splitn(2, "\" ");
        let Some(request_line) = req_parts.next() else { continue; };
        let Some(rest3) = req_parts.next() else { continue; };

        let mut tokens = rest3.split_whitespace();
        let status = tokens.next().and_then(|s| s.parse::<i64>().ok()).unwrap_or(200);
        let size = tokens.next().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);

        let req_tokens: Vec<&str> = request_line.split_whitespace().collect();
        let method = req_tokens.first().unwrap_or(&"GET").to_string();
        let path = req_tokens.get(1).unwrap_or(&"/").to_string();

        let mut user_agent = "-".to_string();
        let mut referer = "-".to_string();

        if let Some(ua_end) = line.rfind('"') {
            if let Some(ua_start) = line[..ua_end].rfind('"') {
                user_agent = line[ua_start + 1..ua_end].to_string();

                let remainder = &line[..ua_start];
                if let Some(ref_end) = remainder.rfind('"') {
                    if let Some(ref_start) = remainder[..ref_end].rfind('"') {
                        referer = remainder[ref_start + 1..ref_end].to_string();
                    }
                }
            }
        }

        let iso_timestamp = parse_nginx_timestamp_to_iso(timestamp);

        let _ = sqlx::query(
            "INSERT INTO log_entries (domain, ip, timestamp, method, path, status, size, referer, user_agent) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(domain)
        .bind(ip.to_string())
        .bind(iso_timestamp)
        .bind(method)
        .bind(path)
        .bind(status)
        .bind(size)
        .bind(referer)
        .bind(user_agent)
        .execute(pool)
        .await;
    }

    let _ = sqlx::query(
        "INSERT INTO log_states (path, inode, last_position) VALUES (?, ?, ?) ON CONFLICT(path) DO UPDATE SET inode = ?, last_position = ?"
    )
    .bind(log_path)
    .bind(current_inode as i64)
    .bind(file_size as i64)
    .bind(current_inode as i64)
    .bind(file_size as i64)
    .execute(pool)
    .await;
}