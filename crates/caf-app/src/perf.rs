//! Runtime performance information for Settings → Performance.
//!
//! Memory is reported as the sum over CAMark's own process and every process it started
//! (WebKitGTK runs the page in separate web and network processes). On Linux each process
//! counts its PSS from `/proc/<pid>/smaps_rollup`: shared pages are split across the
//! processes that map them, so the total neither double-counts nor ignores shared memory.
//! This is the same accounting `camarkctl bench` uses.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUsage {
    /// Sum of PSS over the process tree, in bytes. `None` when this platform has no
    /// accounting wired up, or a process could not be read.
    pub total_bytes: Option<u64>,
    /// Processes counted, including CAMark itself.
    pub process_count: usize,
}

#[cfg(target_os = "linux")]
pub fn memory_usage() -> MemoryUsage {
    let root = std::process::id();
    let pids = linux::descendants(root, &linux::parent_map());
    let total_bytes = pids.iter().try_fold(0u64, |acc, pid| {
        linux::pss_bytes(*pid).map(|b| acc.saturating_add(b))
    });
    MemoryUsage {
        total_bytes,
        process_count: pids.len(),
    }
}

#[cfg(not(target_os = "linux"))]
pub fn memory_usage() -> MemoryUsage {
    MemoryUsage {
        total_bytes: None,
        process_count: 1,
    }
}

#[cfg(any(target_os = "linux", test))]
mod linux {
    use std::collections::HashMap;

    /// `pid -> ppid` for every process readable in `/proc`.
    #[cfg(target_os = "linux")]
    pub fn parent_map() -> HashMap<u32, u32> {
        let mut map = HashMap::new();
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return map;
        };
        for entry in entries.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<u32>().ok())
            else {
                continue;
            };
            if let Some(ppid) = std::fs::read_to_string(format!("/proc/{pid}/stat"))
                .ok()
                .and_then(|stat| parse_ppid(&stat))
            {
                map.insert(pid, ppid);
            }
        }
        map
    }

    /// The parent pid from the text of `/proc/<pid>/stat`. The command name sits in
    /// parentheses and may itself contain spaces or `)`, so fields are read after the
    /// *last* `)`: state, then ppid.
    pub fn parse_ppid(stat: &str) -> Option<u32> {
        let (_, rest) = stat.rsplit_once(')')?;
        let mut fields = rest.split_whitespace();
        let _state = fields.next()?;
        fields.next()?.parse().ok()
    }

    /// `root` plus every process transitively parented by it, `root` first.
    pub fn descendants(root: u32, parents: &HashMap<u32, u32>) -> Vec<u32> {
        let mut out = vec![root];
        let mut i = 0;
        while let Some(&current) = out.get(i) {
            let mut children: Vec<u32> = parents
                .iter()
                .filter(|&(&pid, &ppid)| ppid == current && pid != current)
                .map(|(&pid, _)| pid)
                .collect();
            children.sort_unstable();
            for child in children {
                if !out.contains(&child) {
                    out.push(child);
                }
            }
            i += 1;
        }
        out
    }

    /// PSS in bytes from the text of `/proc/<pid>/smaps_rollup`.
    pub fn parse_pss(rollup: &str) -> Option<u64> {
        rollup.lines().find_map(|line| {
            let kib: u64 = line
                .strip_prefix("Pss:")?
                .trim()
                .trim_end_matches("kB")
                .trim()
                .parse()
                .ok()?;
            Some(kib.saturating_mul(1024))
        })
    }

    #[cfg(target_os = "linux")]
    pub fn pss_bytes(pid: u32) -> Option<u64> {
        parse_pss(&std::fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::linux::{descendants, parse_ppid, parse_pss};
    use std::collections::HashMap;

    #[test]
    fn ppid_is_read_after_the_last_parenthesis() {
        assert_eq!(parse_ppid("1234 (camark) S 1000 1234 1234 0"), Some(1000));
        assert_eq!(parse_ppid("77 (Web Content) (x)) R 42 77 77"), Some(42));
        assert_eq!(parse_ppid("garbage"), None);
    }

    #[test]
    fn pss_line_is_parsed_in_bytes() {
        let rollup =
            "00400000-7ffd rw-p 00000000 00:00 0 [rollup]\nRss:  51200 kB\nPss:  20480 kB\n";
        assert_eq!(parse_pss(rollup), Some(20_480 * 1024));
        assert_eq!(parse_pss("Rss: 1 kB\n"), None);
    }

    #[test]
    fn descendants_walk_the_whole_tree_and_nothing_else() {
        // 10 = CAMark; 11 and 12 its WebKit children; 13 a grandchild; 20 unrelated.
        let parents = HashMap::from([(11, 10), (12, 10), (13, 11), (20, 1), (10, 1)]);
        assert_eq!(descendants(10, &parents), vec![10, 11, 12, 13]);
        assert_eq!(descendants(99, &parents), vec![99]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn own_process_memory_is_measured() {
        let usage = super::memory_usage();
        assert!(usage.process_count >= 1);
        assert!(usage.total_bytes.is_some_and(|b| b > 0));
    }
}
