use anyhow::Result;
use chrono::Utc;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use tor_client;
use tor_dirmgr;
use tor_netdir;
use tor_netdoc::doc::netstatus::RouterFlags;

mod onionoo;

// Header format from dir-list-spec.txt but static so Stem is happy about it. In the future, we
// want to stop doing that.
static HEADER_COMMENT: &'static str =
"/* type=fallback */
/* version=4.0.0 */
/* timestamp=20210412000000 */
/* source=offer-list */";

fn write_file_tor_git(writer: &mut BufWriter<&File>, relay: &tor_netdir::Relay) -> Result<()> {
    let ipv4: Vec<_> = relay
        .rs()
        .orport_addrs()
        .filter(|sa| sa.is_ipv4())
        .collect();
    let ipv6: Vec<_> = relay
        .rs()
        .orport_addrs()
        .filter(|sa| sa.is_ipv6())
        .collect();
    writeln!(
        writer,
        "\"{} orport={} id={}\"",
        ipv4[0].ip(),
        ipv4[0].port(),
        relay.rsa_id().to_string().to_uppercase().replace("$", "")
    )?;
    if !ipv6.is_empty() {
        writeln!(writer, "\" ipv6={}\"", ipv6[0])?;
    }
    writeln!(writer, "/* nickname={} */", relay.rs().nickname())?;
    writeln!(writer, "/* extrainfo=0 */")?;
    writeln!(writer, "/* ===== */")?;
    writeln!(writer, ",")?;
    Ok(())
}

fn write_file_tor_arti(writer: &mut BufWriter<&File>, relay: &tor_netdir::Relay) -> Result<()> {
    writeln!(writer, "    // Nickname: {}", relay.rs().nickname())?;
    writeln!(writer, "    fallback(")?;
    writeln!(
        writer,
        "        \"{}\",",
        relay.rsa_id().to_string().to_uppercase().replace("$", "")
    )?;
    writeln!(
        writer,
        "        \"{}\",",
        relay.md().ed25519_id().to_string()
    )?;
    writeln!(writer, "        &[")?;
    writeln!(
        writer,
        "{: <12}{}", "",
        relay
            .rs()
            .orport_addrs()
            .map(|a| format!("\"{}\"", a))
            .collect::<Vec<String>>()
            .join(",\n            ")
    )?;
    writeln!(writer, "        ],")?;
    writeln!(writer, "    ),")?;
    Ok(())
}

fn write_relay_to_files(
    tor_git_writer: &mut BufWriter<&File>,
    tor_arti_writer: &mut BufWriter<&File>,
    relay: &tor_netdir::Relay,
) -> Result<()> {
    write_file_tor_git(tor_git_writer, relay)?;
    write_file_tor_arti(tor_arti_writer, relay)
}

fn write_header_to_file(writer: &mut BufWriter<&File>) -> Result<()> {
    writeln!(writer, "{}", HEADER_COMMENT)?;
    writeln!(writer, "//")?;
    writeln!(writer, "// Generated on: {}\n", Utc::now().to_rfc2822())?;
    Ok(())
}

fn main() -> Result<()> {
    let mut builder = tor_dirmgr::NetDirConfigBuilder::new();
    builder.use_default_cache_path()?;
    let config: tor_dirmgr::NetDirConfig = builder.finalize()?;

    println!("[+] Fetching onionoo relays...");
    let onionoo_relays_fprs = onionoo::get_relay_fprs_from_onionoo()?;

    tor_rtcompat::task::block_on(async {
        println!("[+] Bootstrapping to the Tor network...");
        let tor_client = Arc::new(tor_client::TorClient::bootstrap(config).await?);
        let netdir = tor_client.dirmgr().netdir();

        println!("[+] Cross-referencing relays between Onionoo and Tor consensus...");

        let relays: Vec<_> = netdir
            .relays()
            .filter(|r| {
                r.is_dir_cache()
                    && r.rs().flags().contains(RouterFlags::FAST)
                    && r.rs().flags().contains(RouterFlags::STABLE)
                    && onionoo_relays_fprs.contains(&r.rsa_id().to_string().to_uppercase())
            })
            .collect();

        println!("Got {} relays. Randomly sampling 200...", relays.len());

        let picks = relays.choose_multiple(&mut rand::thread_rng(), 200);

        // Create files.
        let tor_git_file = File::create("tor-git_fallback_dirs.inc")?;
        let tor_arti_file = File::create("tor-arti_fallback_dirs.inc")?;

        // Create writers.
        let mut tor_git_writer = BufWriter::new(&tor_git_file);
        let mut tor_arti_writer = BufWriter::new(&tor_arti_file);

        // Write header to both files.
        write_header_to_file(&mut tor_git_writer)?;
        write_header_to_file(&mut tor_arti_writer)?;

        // Start the arti file.
        writeln!(tor_arti_writer, "vec![")?;

        for relay in picks {
            write_relay_to_files(&mut tor_git_writer, &mut tor_arti_writer, &relay)?;
        }

        // End the arti file.
        writeln!(tor_arti_writer, "]")?;

        Ok(())
    })
}
