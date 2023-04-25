use semver::VersionReq;

use comfy_table::{Attribute, Cell, Color, Table};

use crate::utils::list_releases_interactively;

pub fn list(version: Option<VersionReq>, count: Option<usize>, all: bool) -> anyhow::Result<()> {
    let mut releases =
        list_releases_interactively().expect("A list of wasmer versions from github.");
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Tags").add_attribute(Attribute::Bold),
        Cell::new("Release").add_attribute(Attribute::Bold),
        Cell::new("Published at").add_attribute(Attribute::Bold),
    ]);
    releases.reverse();
    let release_slice = if let Some(count) = count {
        if count < releases.len() && !all {
            &releases[(releases.len() - count)..]
        } else {
            releases.as_slice()
        }
    } else {
        releases.as_slice()
    };

    for release in release_slice {
        let release_version = release.version();

        if let Some(ref version) = version {
            if !version.matches(&release_version) {
                continue;
            }
        }
        table.add_row(vec![
            Cell::new(release.tags().join(", "))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Italic),
            Cell::new(release_version).add_attribute(Attribute::Bold),
            Cell::new(release.published_time()),
        ]);
    }
    println!("{table}");
    Ok(())
}
