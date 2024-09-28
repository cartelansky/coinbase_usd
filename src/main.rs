use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.exchange.coinbase.com/products";
    let response = reqwest::get(url).await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;

    let mut pairs: Vec<String> = json
        .as_array()
        .ok_or("API yanıtı beklenen formatta değil")?
        .iter()
        .filter_map(|product| {
            let base = product.get("base_currency")?.as_str()?;
            let quote = product.get("quote_currency")?.as_str()?;
            if quote == "USD" {
                Some(format!("COINBASE:{}USD", base))
            } else {
                None
            }
        })
        .collect();

    pairs.sort_by(|a, b| {
        let extract_prefix = |s: &str| -> (String, bool) {
            let prefix = s
                .trim_start_matches("COINBASE:")
                .trim_end_matches("USD")
                .to_string();
            (
                prefix.clone(),
                prefix.chars().next().map_or(false, |c| c.is_ascii_digit()),
            )
        };

        let (a_prefix, a_is_numeric) = extract_prefix(a);
        let (b_prefix, b_is_numeric) = extract_prefix(b);

        match (a_is_numeric, b_is_numeric) {
            (true, true) => b_prefix
                .parse::<u64>()
                .unwrap_or(0)
                .cmp(&a_prefix.parse::<u64>().unwrap_or(0)),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => a.cmp(b),
        }
    });

    let mut file = File::create("coinbase_usd_markets.txt")?;
    for pair in pairs {
        writeln!(file, "{}", pair)?;
    }

    println!("Veriler başarıyla 'coinbase_usd_markets.txt' dosyasına yazıldı.");
    Ok(())
}
