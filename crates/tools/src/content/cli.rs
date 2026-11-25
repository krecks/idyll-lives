use super::*;

// --- The command line --------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Args {
    pub what: String,
    pub ids: Vec<String>,
    pub group: Option<String>,
    pub label: Option<String>,
    pub to: Option<String>,
    pub dry_run: bool,
    pub content: Option<String>,
    pub plan: Option<String>,
}

/// A run's end: what to print to stderr and the exit code.
#[derive(Debug)]
pub struct Exit {
    pub code: i32,
    pub msg: String,
}

fn usage_error(msg: &str) -> Exit {
    let usage = HELP.split("\n\n").next().unwrap();
    Exit {
        code: 2,
        msg: format!("{usage}\ncontent: error: {msg}"),
    }
}

fn fail(msg: impl Into<String>) -> Exit {
    Exit {
        code: 1,
        msg: msg.into(),
    }
}

pub fn parse_args(argv: &[String]) -> Result<Args, Exit> {
    let mut a = Args::default();
    let mut i = 0;
    while i < argv.len() {
        let s = argv[i].as_str();
        i += 1;
        let slot = match s {
            "-h" | "--help" => {
                return Err(Exit {
                    code: 0,
                    msg: String::new(),
                });
            }
            "--dry-run" => {
                a.dry_run = true;
                continue;
            }
            "--group" => &mut a.group,
            "--label" => &mut a.label,
            "--to" => &mut a.to,
            "--content" => &mut a.content,
            "--plan" => &mut a.plan,
            _ if s.starts_with('-') => return Err(usage_error(&format!("unknown option: {s}"))),
            _ if a.what.is_empty() => {
                if !["list", "in", "out"].contains(&s) {
                    return Err(usage_error(&format!("list, in or out, not '{s}'")));
                }
                a.what = s.to_string();
                continue;
            }
            _ => {
                a.ids.push(s.to_string());
                continue;
            }
        };
        let Some(v) = argv.get(i) else {
            return Err(usage_error(&format!("{s} needs a value")));
        };
        *slot = Some(v.clone());
        i += 1;
    }
    if a.what.is_empty() {
        return Err(usage_error("list, in or out?"));
    }
    Ok(a)
}

/// The value of key `k` in `v`, in the order the content has them.
fn find<'a, V>(v: &'a [(String, V)], k: &str) -> Option<&'a V> {
    v.iter().find(|e| e.0 == k).map(|e| &e.1)
}

/// Sets key `k` to `x`: a key seen before keeps its place and takes the new value (the last file wins).
fn put<V>(v: &mut Vec<(String, V)>, k: String, x: V) {
    match v.iter_mut().find(|e| e.0 == k) {
        Some(e) => e.1 = x,
        None => v.push((k, x)),
    }
}

fn opt(s: &Option<String>) -> String {
    s.clone().unwrap_or_else(|| "None".into())
}

fn dash(s: &Option<String>) -> String {
    s.clone().unwrap_or_else(|| "—".into())
}

/// Runs the tool; stdout goes to `out`.
pub fn run(a: &Args, out: &mut dyn std::io::Write) -> Result<(), Exit> {
    let root = crate::repo_root();
    let content = a
        .content
        .as_deref()
        .map_or_else(|| root.join("web/public/content"), py_path);
    let plan = a
        .plan
        .as_deref()
        .map_or_else(|| root.join("docs/objects"), py_path);
    let files = load(&content).map_err(fail)?;
    let mut eds: Vec<Editor> = files.iter().map(|f| Editor::new(&f.text)).collect();
    let mut by_id: Vec<(String, (usize, &Node))> = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        for o in f.array("objects") {
            if let Some(id) = o.get("id") {
                put(
                    &mut by_id,
                    id.value.py().unwrap_or_else(|| "None".into()),
                    (fi, o),
                );
            }
        }
    }
    let mut cats: Vec<(String, (usize, &Node))> = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        for c in f.array("buyCategories") {
            put(
                &mut cats,
                c.val("id").unwrap_or_else(|| "None".into()),
                (fi, c),
            );
        }
    }
    let Some(&(vfile, vcat)) = find(&cats, "validate") else {
        return Err(fail(
            "validate: there is no Validate category in the content",
        ));
    };
    let vgroups = vcat.get("groups");
    let vitems: &[Node] = vgroups.map_or(&[], |g| &g.items);
    let where_ = |o: &Node| (o.val("category"), o.val("group"));
    let groups_of = |cat: &str| -> Vec<(String, &Node)> {
        find(&cats, cat)
            .and_then(|(_, c)| c.get("groups"))
            .map_or(Vec::new(), |arr| {
                arr.items
                    .iter()
                    .map(|g| (g.val("id").unwrap_or_else(|| "None".into()), g))
                    .collect()
            })
    };
    let homes = if plan.is_dir() {
        back_to_table(&plan).map_err(fail)?
    } else {
        HashMap::new()
    };
    let base_data: serde_json::Value =
        serde_json::from_str(&files[0].text).map_err(|e| fail(e.to_string()))?;
    let free_cats: HashSet<String> = base_data
        .get("objectRules")
        .and_then(|r| r.get("freeRotation"))
        .and_then(|f| f.as_array())
        .map_or(HashSet::new(), |a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
    let w = |out: &mut dyn std::io::Write, s: String| {
        let _ = writeln!(out, "{s}");
    };

    if a.what == "list" {
        let mut in_validate: Vec<(String, Vec<String>)> = Vec::new();
        for (id, (_, o)) in &by_id {
            let (cat, group) = where_(o);
            if cat.as_deref() == Some("validate") {
                let g = opt(&group);
                match in_validate.iter_mut().find(|e| e.0 == g) {
                    Some(e) => e.1.push(id.clone()),
                    None => in_validate.push((g, vec![id.clone()])),
                }
            }
        }
        for g in vitems {
            let gid = opt(&g.val("id"));
            let items = match in_validate.iter().position(|e| e.0 == gid) {
                Some(k) => in_validate.remove(k).1,
                None => Vec::new(),
            };
            w(out, format!("{gid} ({})", items.len()));
            for id in items {
                let home = match homes.get(&id) {
                    Some((c, g)) => format!("{c} / {}", dash(g)),
                    None => "(no Home in docs/objects/)".into(),
                };
                w(out, format!("  {id}  ->  {home}"));
            }
        }
        for (gid, items) in &in_validate {
            w(
                out,
                format!("{gid} (not a Validate group!): {}", items.join(", ")),
            );
        }
        return Ok(());
    }

    let mut ids: Vec<String> = a.ids.clone();
    if a.what == "out"
        && let Some(group) = &a.group
    {
        for (id, (_, o)) in &by_id {
            if where_(o) == (Some("validate".into()), Some(group.clone())) && !ids.contains(id) {
                ids.push(id.clone());
            }
        }
    }
    if ids.is_empty() {
        return Err(fail("validate: which items? (ids, or --group with out)"));
    }
    let missing: Vec<&str> = ids
        .iter()
        .filter(|i| find(&by_id, i).is_none())
        .map(|s| s.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(fail(format!(
            "validate: no such item: {}",
            missing.join(", ")
        )));
    }

    let mut expect: HashMap<String, (String, Option<String>)> = HashMap::new();
    if a.what == "in" {
        let Some(group) = &a.group else {
            return Err(fail("validate: in needs --group <validate group>"));
        };
        let existing: Vec<String> = groups_of("validate").into_iter().map(|(k, _)| k).collect();
        let mut outside = false;
        for id in &ids {
            let (fi, o) = *find(&by_id, id).unwrap();
            let (cat, g) = where_(o);
            if cat.as_deref() == Some("validate") {
                return Err(fail(format!(
                    "validate: {id} is in Validate already ({})",
                    opt(&g)
                )));
            }
            let Some(cat) = cat.clone().filter(|c| find(&cats, c).is_some()) else {
                return Err(fail(format!(
                    "validate: {id} has no known category ({})",
                    opt(&cat)
                )));
            };
            let home_group = g.as_ref().and_then(|g| {
                groups_of(&cat)
                    .into_iter()
                    .find(|(k, _)| k == g)
                    .map(|(_, n)| n)
            });
            outside |= home_group
                .and_then(|h| h.get("outside"))
                .is_some_and(|n| n.value.truthy());
            let ed = &mut eds[fi];
            ed.replace(o.get("category").unwrap(), "\"validate\"");
            ed.set_member(o, "group", &json_str(group), Some("category"));
            if o.get("freeRotation").is_none()
                && free_cats.contains(&cat)
                && !free_cats.contains("validate")
            {
                ed.set_member(o, "freeRotation", "true", Some("category"));
            }
            w(
                out,
                format!("{id}: {cat} / {} -> validate / {group}", dash(&g)),
            );
        }
        if !existing.contains(group) {
            let label = json_str(&a.label.clone().unwrap_or_else(|| label_of(group)));
            let gid = json_str(group);
            let extra = if outside { ", \"outside\": true" } else { "" };
            let Some(vgroups) = vgroups else {
                return Err(fail("validate: the Validate category has no \"groups\""));
            };
            eds[vfile].append_item(
                vgroups,
                &format!("{{\"id\": {gid}, \"label\": {label}{extra}}}"),
                |ind| {
                    format!(
                        "{{\n{ind}  \"id\": {gid},\n{ind}  \"label\": {label}{}\n{ind}}}",
                        if outside {
                            format!(",\n{ind}  \"outside\": true")
                        } else {
                            String::new()
                        }
                    )
                },
            );
            w(
                out,
                format!(
                    "new Validate group: {group}{}",
                    if outside { " (outside)" } else { "" }
                ),
            );
        }
        for id in &ids {
            expect.insert(id.clone(), ("validate".into(), Some(group.clone())));
        }
    } else {
        let mut emptied: Vec<Option<String>> = Vec::new();
        let plan_s = plan.display().to_string();
        for id in &ids {
            let (fi, o) = *find(&by_id, id).unwrap();
            let (cat, group) = where_(o);
            if cat.as_deref() != Some("validate") {
                return Err(fail(format!(
                    "validate: {id} is not in Validate ({} / {})",
                    opt(&cat),
                    opt(&group)
                )));
            }
            let home = if let Some(to) = &a.to {
                let (c, g) = to.split_once('/').unwrap_or((to, ""));
                let g = g.trim();
                (c.trim().to_string(), (!g.is_empty()).then(|| g.to_string()))
            } else if let Some(h) = homes.get(id) {
                h.clone()
            } else {
                return Err(fail(format!(
                    "validate: no \"Home\" for {id} in {plan_s}: give --to <category>/<group>"
                )));
            };
            if find(&cats, &home.0).is_none() {
                return Err(fail(format!("validate: {id}: no category \"{}\"", home.0)));
            }
            if let Some(hg) = &home.1 {
                let groups = groups_of(&home.0);
                if !groups.iter().any(|(k, _)| k == hg) {
                    let names: Vec<&str> = groups.iter().map(|(k, _)| k.as_str()).collect();
                    return Err(fail(format!(
                        "validate: {id}: category \"{}\" has no group \"{hg}\" ({})",
                        home.0,
                        names.join(", ")
                    )));
                }
            }
            let ed = &mut eds[fi];
            ed.replace(o.get("category").unwrap(), &json_str(&home.0));
            match &home.1 {
                None => ed.remove_member(o, "group"),
                Some(hg) => ed.set_member(o, "group", &json_str(hg), Some("category")),
            }
            if !emptied.contains(&group) {
                emptied.push(group.clone());
            }
            w(
                out,
                format!(
                    "{id}: validate / {} -> {} / {}",
                    opt(&group),
                    home.0,
                    dash(&home.1)
                ),
            );
            expect.insert(id.clone(), home);
        }
        // Validate groups nothing is left in.
        let still: HashSet<Option<String>> = by_id
            .iter()
            .filter(|(i, (_, o))| {
                !expect.contains_key(i) && where_(o).0.as_deref() == Some("validate")
            })
            .map(|(_, (_, o))| where_(o).1)
            .collect();
        let dropped: Vec<usize> = (0..vitems.len())
            .filter(|&k| {
                let gid = vitems[k].val("id");
                emptied.contains(&gid) && !still.contains(&gid)
            })
            .collect();
        for &k in &dropped {
            eds[vfile].remove_item(vgroups.unwrap(), k);
            w(
                out,
                format!(
                    "dropped the empty Validate group: {}",
                    opt(&vitems[k].val("id"))
                ),
            );
        }
        if dropped.len() == vitems.len() {
            w(
                out,
                "Validate is empty now: remove the category and its icon.".into(),
            );
        }
    }

    // Build, parse and check every changed file before writing any.
    let mut done: Vec<(usize, String)> = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        if eds[fi].edits.is_empty() {
            continue;
        }
        let text = eds[fi].new_text(&f.path).map_err(fail)?;
        let data: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| fail(format!("{}: {e}", f.path.display())))?;
        for o in data
            .get("objects")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v)
        {
            let Some(id) = o.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            if let Some((cat, group)) = expect.get(id) {
                let got = (
                    o.get("category").and_then(|v| v.as_str()),
                    o.get("group").and_then(|v| v.as_str()),
                );
                if got != (Some(cat.as_str()), group.as_deref()) {
                    return Err(fail(format!(
                        "{id}: {} / {}",
                        got.0.unwrap_or("None"),
                        got.1.unwrap_or("None")
                    )));
                }
            }
        }
        done.push((fi, text));
    }
    for (fi, text) in done {
        let f = &files[fi];
        let (rel, header) = match f.path.strip_prefix(&root) {
            Ok(r) => (r.display().to_string(), format!("a/{}", r.display())),
            Err(_) if f.path.is_relative() => (
                f.path.display().to_string(),
                format!("a/{}", f.path.display()),
            ),
            Err(_) => (f.path.display().to_string(), f.path.display().to_string()),
        };
        if a.dry_run {
            let header_b = if header.starts_with("a/") {
                format!("b/{rel}")
            } else {
                header.clone()
            };
            let diff = similar::TextDiff::configure()
                .algorithm(similar::Algorithm::Myers)
                .diff_lines(f.text.as_str(), text.as_str());
            let _ = write!(
                out,
                "{}",
                diff.unified_diff()
                    .context_radius(3)
                    .missing_newline_hint(false)
                    .header(&header, &header_b)
            );
        } else {
            std::fs::write(&f.path, &text)
                .map_err(|e| fail(format!("{}: {e}", f.path.display())))?;
            w(out, format!("wrote {rel}"));
        }
    }
    Ok(())
}

/// The binary's entry: parses `argv` (without the program name), runs, and returns the exit code.
pub fn main_with(argv: &[String]) -> i32 {
    let mut stdout = std::io::stdout().lock();
    let res = parse_args(argv).and_then(|a| run(&a, &mut stdout));
    let _ = stdout.flush();
    match res {
        Ok(()) => 0,
        Err(e) if e.code == 0 => {
            print!("{HELP}");
            0
        }
        Err(e) => {
            eprintln!("{}", e.msg);
            e.code
        }
    }
}
