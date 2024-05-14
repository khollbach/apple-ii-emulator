use std::fs;

use anyhow::{Context, Result};
use itertools::Itertools;
use scraper::{Html, Selector};

fn main() -> Result<()> {
    let html = fs::read_to_string("opcode-table.html")?;
    let table = Html::parse_document(&html);
    let selector = Selector::parse("td").unwrap();

    for (idx, cell) in table.select(&selector).enumerate() {
        let (t,) = cell
            .text()
            .collect_tuple()
            .context("expected exactly one text node")?;
        if t == "---" {
            continue;
        }
        println!("{idx} {t}");
    }

    Ok(())
}
