use rusqlite::Connection;

/// Default cybersecurity news sources seeded on first run.
struct DefaultSource {
    name: &'static str,
    feed_url: &'static str,
    feed_type: &'static str,
    default_tag: &'static str,
}

const DEFAULT_SOURCES: &[DefaultSource] = &[
    // General cybersecurity news
    DefaultSource {
        name: "BleepingComputer",
        feed_url: "https://www.bleepingcomputer.com/feed/",
        feed_type: "rss",
        default_tag: "general",
    },
    DefaultSource {
        name: "Krebs on Security",
        feed_url: "https://krebsonsecurity.com/feed/",
        feed_type: "rss",
        default_tag: "general",
    },
    DefaultSource {
        name: "The Hacker News",
        feed_url: "https://feeds.feedburner.com/TheHackersNews",
        feed_type: "rss",
        default_tag: "general",
    },
    DefaultSource {
        name: "DarkReading",
        feed_url: "https://www.darkreading.com/rss.xml",
        feed_type: "rss",
        default_tag: "general",
    },
    DefaultSource {
        name: "SecurityWeek",
        feed_url: "https://www.securityweek.com/feed/",
        feed_type: "rss",
        default_tag: "general",
    },
    DefaultSource {
        name: "Ars Technica Security",
        feed_url: "https://feeds.arstechnica.com/arstechnica/security",
        feed_type: "rss",
        default_tag: "general",
    },
    // Microsoft-specific
    DefaultSource {
        name: "Microsoft Security Blog",
        feed_url: "https://www.microsoft.com/en-us/security/blog/feed/",
        feed_type: "rss",
        default_tag: "microsoft",
    },
    DefaultSource {
        name: "Microsoft Security Response Center",
        feed_url: "https://msrc.microsoft.com/blog/feed/",
        feed_type: "rss",
        default_tag: "microsoft",
    },
    // Threat Intelligence
    DefaultSource {
        name: "CISA Alerts",
        feed_url: "https://www.cisa.gov/cybersecurity-advisories/all.xml",
        feed_type: "rss",
        default_tag: "threat_intel",
    },
    DefaultSource {
        name: "US-CERT Current Activity",
        feed_url: "https://www.cisa.gov/news.xml",
        feed_type: "rss",
        default_tag: "threat_intel",
    },
];

/// Seed default sources if the news_sources table is empty.
pub fn seed_default_sources(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM news_sources WHERE is_default = 1",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        log::info!("Default sources already seeded ({count} found), skipping.");
        return Ok(());
    }

    let insert_sql = "INSERT OR IGNORE INTO news_sources (name, feed_url, feed_type, default_tag, is_default, is_active) VALUES (?1, ?2, ?3, ?4, 1, 1)";

    for source in DEFAULT_SOURCES {
        conn.execute(
            insert_sql,
            rusqlite::params![source.name, source.feed_url, source.feed_type, source.default_tag],
        )?;
    }

    log::info!("Seeded {} default news sources.", DEFAULT_SOURCES.len());
    Ok(())
}
