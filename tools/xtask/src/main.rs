use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const RELEASE_PACKAGES: &[(&str, &str)] = &[
    ("terrakit-core", "engine/terrakit-core"),
    ("terrakit-pipeline", "engine/terrakit-pipeline"),
    ("terrakit-runtime", "engine/terrakit-runtime"),
    ("terrakit-algorithms", "generation/terrakit-algorithms"),
    ("terrakit-builtins", "generation/terrakit-builtins"),
    ("terrakit-c-api", "engine/terrakit-c-api"),
    ("terrakit-console", "interfaces/terrakit-console"),
];

const REQUIRED_C_SYMBOLS: &[&str] = &[
    "tk_get_abi_version",
    "tk_stage_registry_create_builtin",
    "tk_pipeline_assembler_finish",
    "tk_runtime_generate_2d",
    "tk_generation_result_get_mesh",
];

fn main() {
    if let Err(error) = try_main() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().cloned() else {
        return usage();
    };
    args.remove(0);

    match command.as_str() {
        "archive-c-package" => archive_c_package(args),
        "check-release-manifests" => check_release_manifests(args),
        "check-tag-version" => check_tag_version(args),
        "c-symbol-check" => c_symbol_check(args),
        "doc-check" => doc_check(args),
        "ensure-cbindgen" => ensure_cbindgen(args),
        "host-target" => host_target(args),
        "mkdir" => make_dirs(args),
        "package-c-api" => package_c_api(args),
        "run-with-library-path" => run_with_library_path(args),
        _ => usage(),
    }
}

fn usage() -> Result<()> {
    fail(
        "usage: cargo run -p terrakit-xtask -- <archive-c-package|check-release-manifests|check-tag-version|c-symbol-check|doc-check|ensure-cbindgen|host-target|mkdir|package-c-api|run-with-library-path>",
    )
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask should live under tools/xtask")
        .to_path_buf()
}

fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(io::Error::other(message.into()).into())
}

fn require_no_args(args: &[String]) -> Result<()> {
    if args.is_empty() {
        Ok(())
    } else {
        fail(format!("unexpected arguments: {}", args.join(" ")))
    }
}

fn take_option(args: &mut Vec<String>, name: &str) -> Result<Option<String>> {
    let Some(index) = args.iter().position(|arg| arg == name) else {
        return Ok(None);
    };
    if index + 1 >= args.len() {
        return fail(format!("missing value for {name}"));
    }
    let value = args.remove(index + 1);
    args.remove(index);
    Ok(Some(value))
}

fn absolute_from_root(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root().join(path)
    }
}

fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|error| {
        io::Error::other(format!("failed to read {}: {error}", path.display())).into()
    })
}

fn run_command(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        fail(format!("command failed with {status}: {command:?}"))
    }
}

fn command_output(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return fail(format!(
            "command failed with {}: {command:?}\n{}",
            output.status,
            stderr.trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn github_output(name: &str, value: &str) -> Result<()> {
    let Some(output_path) = env::var_os("GITHUB_OUTPUT") else {
        return Ok(());
    };

    let mut output = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)?;
    writeln!(output, "{name}={value}")?;
    Ok(())
}

fn workspace_version() -> Result<String> {
    let root_manifest = repo_root().join("Cargo.toml");
    let contents = read_to_string(&root_manifest)?;
    toml_string_field(&contents, "workspace.package", "version").ok_or_else(|| {
        io::Error::other("could not find [workspace.package] version in Cargo.toml").into()
    })
}

fn check_tag_version(mut args: Vec<String>) -> Result<()> {
    let ref_name = if args.is_empty() {
        env::var("GITHUB_REF_NAME")
            .or_else(|_| env::var("GITHUB_REF").map(|ref_name| last_ref_segment(&ref_name)))
            .unwrap_or_default()
    } else {
        args.remove(0)
    };
    require_no_args(&args)?;

    if ref_name.is_empty() {
        return fail("no tag or ref name was provided");
    }

    let tag_version = ref_name.strip_prefix('v').unwrap_or(&ref_name);
    let manifest_version = workspace_version()?;

    github_output("tag", &ref_name)?;
    github_output("version", &manifest_version)?;

    if tag_version != manifest_version {
        return fail(format!(
            "tag {ref_name} does not match workspace version {manifest_version}; use tag v{manifest_version}"
        ));
    }

    println!("Release tag {ref_name} matches workspace version {manifest_version}.");
    Ok(())
}

fn last_ref_segment(ref_name: &str) -> String {
    ref_name
        .rsplit_once('/')
        .map_or(ref_name, |(_, segment)| segment)
        .to_string()
}

fn host_target(args: Vec<String>) -> Result<()> {
    require_no_args(&args)?;

    let mut command = Command::new("rustc");
    command.arg("-vV");
    let output = command_output(&mut command)?;
    let host = output
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| io::Error::other("rustc -vV did not report a host triple"))?;

    github_output("target", host)?;
    println!("{host}");
    Ok(())
}

fn doc_check(args: Vec<String>) -> Result<()> {
    require_no_args(&args)?;

    let mut command = Command::new("cargo");
    command
        .arg("doc")
        .arg("--workspace")
        .arg("--all-features")
        .arg("--no-deps")
        .env("RUSTDOCFLAGS", "-D warnings");
    run_command(&mut command)
}

fn ensure_cbindgen(mut args: Vec<String>) -> Result<()> {
    let version = take_option(&mut args, "--version")?.unwrap_or_else(|| "0.29.0".to_string());
    let binary = take_option(&mut args, "--binary")?.unwrap_or_else(|| "cbindgen".to_string());
    require_no_args(&args)?;

    if command_exists(&binary) {
        return Ok(());
    }

    let mut command = Command::new("cargo");
    command
        .arg("install")
        .arg("cbindgen")
        .arg("--version")
        .arg(version)
        .arg("--locked");
    run_command(&mut command)
}

fn make_dirs(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return fail("mkdir requires at least one path");
    }

    for path in args {
        fs::create_dir_all(path)?;
    }

    Ok(())
}

fn run_with_library_path(args: Vec<String>) -> Result<()> {
    let Some(separator) = args.iter().position(|arg| arg == "--") else {
        return fail("run-with-library-path requires -- before the command");
    };

    let mut options = args[..separator].to_vec();
    let command_args = args[separator + 1..].to_vec();
    let target_dir =
        take_option(&mut options, "--target-dir")?.unwrap_or_else(|| "target/debug".to_string());
    require_no_args(&options)?;

    let Some(program) = command_args.first() else {
        return fail("run-with-library-path requires a command to run");
    };

    let target_dir = absolute_from_root(target_dir);
    let mut command = Command::new(program);
    command.args(&command_args[1..]);
    prepend_path_env(&mut command, "PATH", &target_dir)?;
    prepend_path_env(&mut command, "LD_LIBRARY_PATH", &target_dir)?;
    prepend_path_env(&mut command, "DYLD_LIBRARY_PATH", &target_dir)?;
    run_command(&mut command)
}

fn prepend_path_env(command: &mut Command, name: &str, path: &Path) -> Result<()> {
    let mut paths = vec![path.to_path_buf()];
    if let Some(existing) = env::var_os(name) {
        paths.extend(env::split_paths(&existing));
    }
    command.env(name, env::join_paths(paths)?);
    Ok(())
}

fn c_symbol_check(mut args: Vec<String>) -> Result<()> {
    let target_dir =
        take_option(&mut args, "--target-dir")?.unwrap_or_else(|| "target/debug".to_string());
    let symbols = take_option(&mut args, "--symbols")?
        .map(|symbols| {
            symbols
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            REQUIRED_C_SYMBOLS
                .iter()
                .map(|symbol| symbol.to_string())
                .collect()
        });
    require_no_args(&args)?;

    let target_dir = absolute_from_root(target_dir);
    let output = exported_symbols_output(&target_dir)?;
    let missing = symbols
        .iter()
        .filter(|symbol| !output.contains(symbol.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    if missing.is_empty() {
        println!("exported symbols OK: {}", symbols.join(", "));
        Ok(())
    } else {
        fail(format!("missing exported symbols: {}", missing.join(", ")))
    }
}

#[cfg(target_os = "windows")]
fn exported_symbols_output(target_dir: &Path) -> Result<String> {
    let library = target_dir.join("terrakit.dll");
    if command_exists("dumpbin") {
        let mut command = Command::new("dumpbin");
        command.arg("/exports").arg(library);
        command_output(&mut command)
    } else {
        let mut command = Command::new("objdump");
        command.arg("-p").arg(library);
        command_output(&mut command)
    }
}

#[cfg(target_os = "macos")]
fn exported_symbols_output(target_dir: &Path) -> Result<String> {
    let mut command = Command::new("nm");
    command.arg("-gU").arg(target_dir.join("libterrakit.dylib"));
    command_output(&mut command)
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn exported_symbols_output(target_dir: &Path) -> Result<String> {
    let mut command = Command::new("nm");
    command
        .arg("-D")
        .arg("--defined-only")
        .arg(target_dir.join("libterrakit.so"));
    command_output(&mut command)
}

fn package_c_api(mut args: Vec<String>) -> Result<()> {
    let target_dir =
        take_option(&mut args, "--target-dir")?.unwrap_or_else(|| "target/debug".to_string());
    let target_triple =
        take_option(&mut args, "--target-triple")?.unwrap_or_else(|| "native".to_string());
    require_no_args(&args)?;

    let root = repo_root();
    let target_dir = absolute_from_root(target_dir);
    let package_dir = root.join("dist").join("c").join(&target_triple);

    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)?;
    }

    let include_dir = package_dir.join("include");
    let lib_dir = package_dir.join("lib");
    let examples_dir = package_dir.join("examples");
    let docs_dir = package_dir.join("docs");

    let mut copied_artifacts = Vec::new();
    for artifact in [
        "terrakit.dll",
        "terrakit.dll.lib",
        "terrakit.lib",
        "libterrakit.dll.a",
        "libterrakit.so",
        "libterrakit.dylib",
        "libterrakit.a",
    ] {
        let artifact_path = target_dir.join(artifact);
        if artifact_path.exists() {
            copied_artifacts.push(copy_file(&artifact_path, &lib_dir.join(artifact))?);
        }
    }

    if copied_artifacts.is_empty() {
        return fail(format!(
            "no C ABI artifacts found in {}",
            target_dir.display()
        ));
    }

    let copied_examples = vec![
        copy_file(
            &root
                .join("bindings")
                .join("c")
                .join("examples")
                .join("generate_heightfield.c"),
            &examples_dir.join("generate_heightfield.c"),
        )?,
        copy_file(
            &root
                .join("bindings")
                .join("c")
                .join("examples")
                .join("header_smoke.cpp"),
            &examples_dir.join("header_smoke.cpp"),
        )?,
    ];

    let copied_docs = vec![
        copy_file(
            &root.join("bindings").join("c").join("README.md"),
            &docs_dir.join("C-ABI.md"),
        )?,
        copy_file(&root.join("README.md"), &package_dir.join("README.md"))?,
    ];

    let copied_include = copy_file(
        &root
            .join("bindings")
            .join("c")
            .join("include")
            .join("terrakit.h"),
        &include_dir.join("terrakit.h"),
    )?;

    write_package_manifest(
        &package_dir,
        &target_triple,
        &target_dir,
        &copied_include,
        &copied_artifacts,
        &copied_examples,
        &copied_docs,
    )?;

    println!("{}", package_dir.display());
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<String> {
    let parent = destination.parent().ok_or_else(|| {
        io::Error::other(format!(
            "destination has no parent directory: {}",
            destination.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    fs::copy(source, destination)?;
    destination
        .file_name()
        .and_then(OsStr::to_str)
        .map(str::to_string)
        .ok_or_else(|| io::Error::other("copied file name was not valid UTF-8").into())
}

fn write_package_manifest(
    package_dir: &Path,
    target_triple: &str,
    target_dir: &Path,
    include: &str,
    libraries: &[String],
    examples: &[String],
    docs: &[String],
) -> Result<()> {
    let manifest = format!(
        concat!(
            "{{\n",
            "  \"package\": \"terrakit-c-api\",\n",
            "  \"version\": {},\n",
            "  \"target_triple\": {},\n",
            "  \"target_dir\": {},\n",
            "  \"include\": [{}],\n",
            "  \"libraries\": {},\n",
            "  \"examples\": {},\n",
            "  \"docs\": {}\n",
            "}}\n"
        ),
        json_string(&workspace_version()?),
        json_string(target_triple),
        json_string(&target_dir.to_string_lossy()),
        json_string(include),
        json_string_array(libraries),
        json_string_array(examples),
        json_string_array(docs),
    );

    fs::write(package_dir.join("package-manifest.json"), manifest)?;
    Ok(())
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>();
    format!("[{}]", values.join(", "))
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn archive_c_package(mut args: Vec<String>) -> Result<()> {
    let target_triple =
        take_option(&mut args, "--target-triple")?.unwrap_or_else(|| "native".to_string());
    let package_dir = take_option(&mut args, "--package-dir")?
        .map(absolute_from_root)
        .unwrap_or_else(|| repo_root().join("dist").join("c").join(&target_triple));
    require_no_args(&args)?;

    if !package_dir.is_dir() {
        return fail(format!(
            "package directory does not exist: {}",
            package_dir.display()
        ));
    }

    let version = workspace_version()?;
    let archive_dir = repo_root().join("dist").join("archives");
    fs::create_dir_all(&archive_dir)?;

    let archive_name = format!("terrakit-c-api-{version}-{target_triple}.zip");
    let archive_path = archive_dir.join(&archive_name);
    let top_level = format!("terrakit-c-api-{version}-{target_triple}");

    if archive_path.exists() {
        fs::remove_file(&archive_path)?;
    }

    let files = collect_files(&package_dir)?;
    if files.is_empty() {
        return fail(format!(
            "no files found to archive under {}",
            package_dir.display()
        ));
    }

    write_zip_archive(&archive_path, &package_dir, &top_level, &files)?;

    let digest = sha256_hex(&fs::read(&archive_path)?);
    let checksum_path = archive_dir.join(format!("{archive_name}.sha256"));
    fs::write(&checksum_path, format!("{digest}  {archive_name}\n"))?;

    github_output("archive", &archive_path.to_string_lossy())?;
    github_output("checksum", &checksum_path.to_string_lossy())?;

    println!("{}", archive_path.display());
    println!("{}", checksum_path.display());
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_into(root, &mut files)?;
    files.sort_by(|left, right| {
        let left_name = zip_relative_name(root, left);
        let right_name = zip_relative_name(root, right);
        left_name.cmp(&right_name)
    });
    Ok(files)
}

fn collect_files_into(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_into(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn write_zip_archive(
    archive_path: &Path,
    package_dir: &Path,
    top_level: &str,
    files: &[PathBuf],
) -> Result<()> {
    let mut archive = File::create(archive_path)?;
    let mut central_directory = Vec::new();

    for path in files {
        let data = fs::read(path)?;
        let crc = crc32(&data);
        let size = u32::try_from(data.len())?;
        let offset = u32::try_from(archive.stream_position()?)?;
        let entry_name = format!("{top_level}/{}", zip_relative_name(package_dir, path));
        let entry_name = entry_name.into_bytes();
        let name_len = u16::try_from(entry_name.len())?;

        write_u32(&mut archive, 0x0403_4b50)?;
        write_u16(&mut archive, 20)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, dos_date_1980())?;
        write_u32(&mut archive, crc)?;
        write_u32(&mut archive, size)?;
        write_u32(&mut archive, size)?;
        write_u16(&mut archive, name_len)?;
        write_u16(&mut archive, 0)?;
        archive.write_all(&entry_name)?;
        archive.write_all(&data)?;

        central_directory.push(CentralDirectoryEntry {
            name: entry_name,
            crc,
            size,
            offset,
        });
    }

    let central_start = u32::try_from(archive.stream_position()?)?;
    for entry in &central_directory {
        let name_len = u16::try_from(entry.name.len())?;

        write_u32(&mut archive, 0x0201_4b50)?;
        write_u16(&mut archive, 20)?;
        write_u16(&mut archive, 20)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, dos_date_1980())?;
        write_u32(&mut archive, entry.crc)?;
        write_u32(&mut archive, entry.size)?;
        write_u32(&mut archive, entry.size)?;
        write_u16(&mut archive, name_len)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u16(&mut archive, 0)?;
        write_u32(&mut archive, 0)?;
        write_u32(&mut archive, entry.offset)?;
        archive.write_all(&entry.name)?;
    }

    let central_end = u32::try_from(archive.stream_position()?)?;
    let central_size = central_end - central_start;
    let entry_count = u16::try_from(central_directory.len())?;

    write_u32(&mut archive, 0x0605_4b50)?;
    write_u16(&mut archive, 0)?;
    write_u16(&mut archive, 0)?;
    write_u16(&mut archive, entry_count)?;
    write_u16(&mut archive, entry_count)?;
    write_u32(&mut archive, central_size)?;
    write_u32(&mut archive, central_start)?;
    write_u16(&mut archive, 0)?;

    Ok(())
}

struct CentralDirectoryEntry {
    name: Vec<u8>,
    crc: u32,
    size: u32,
    offset: u32,
}

fn zip_relative_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn dos_date_1980() -> u16 {
    (1 << 5) | 1
}

fn write_u16(writer: &mut impl Write, value: u16) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn write_u32(writer: &mut impl Write, value: u32) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn sha256_hex(bytes: &[u8]) -> String {
    sha256(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    let mut hash: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let bit_len = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (index, word) in chunk.chunks_exact(4).take(16).enumerate() {
            schedule[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }

        let mut a = hash[0];
        let mut b = hash[1];
        let mut c = hash[2];
        let mut d = hash[3];
        let mut e = hash[4];
        let mut f = hash[5];
        let mut g = hash[6];
        let mut h = hash[7];

        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(schedule[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sum0.wrapping_add(majority);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        hash[0] = hash[0].wrapping_add(a);
        hash[1] = hash[1].wrapping_add(b);
        hash[2] = hash[2].wrapping_add(c);
        hash[3] = hash[3].wrapping_add(d);
        hash[4] = hash[4].wrapping_add(e);
        hash[5] = hash[5].wrapping_add(f);
        hash[6] = hash[6].wrapping_add(g);
        hash[7] = hash[7].wrapping_add(h);
    }

    let mut output = [0u8; 32];
    for (index, word) in hash.iter().enumerate() {
        output[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    output
}

fn check_release_manifests(args: Vec<String>) -> Result<()> {
    require_no_args(&args)?;

    let root = repo_root();
    let root_manifest = root.join("Cargo.toml");
    let root_manifest_contents = read_to_string(&root_manifest)?;
    let version = workspace_version()?;
    let release_package_names = RELEASE_PACKAGES
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();

    for (name, manifest_path) in discover_workspace_packages()? {
        if !release_package_names.contains(&name.as_str()) {
            return fail(format!(
                "workspace package {name} is missing from release manifest checks: {}",
                manifest_path.display()
            ));
        }
    }

    for (package_name, relative_dir) in RELEASE_PACKAGES {
        let manifest_dir = root.join(relative_dir);
        let manifest_path = manifest_dir.join("Cargo.toml");
        let contents = read_to_string(&manifest_path)?;

        let actual_name = toml_string_field(&contents, "package", "name")
            .ok_or_else(|| io::Error::other(format!("{package_name} is missing package.name")))?;
        if actual_name != *package_name {
            return fail(format!(
                "expected package {package_name}, found {actual_name} in {}",
                manifest_path.display()
            ));
        }

        check_package_version(package_name, &contents, &version)?;
        check_package_description(package_name, &contents, &root_manifest_contents)?;
        check_package_readme(
            package_name,
            &contents,
            &manifest_dir,
            &root_manifest_contents,
        )?;
        check_package_dependency_versions(
            package_name,
            &contents,
            &root_manifest_contents,
            &version,
        )?;
    }

    println!(
        "Release manifests are version-consistent: {}",
        release_package_names.join(", ")
    );
    Ok(())
}

fn discover_workspace_packages() -> Result<Vec<(String, PathBuf)>> {
    let root = repo_root();
    let root_manifest_contents = read_to_string(&root.join("Cargo.toml"))?;
    let mut packages = Vec::new();

    for pattern in workspace_member_patterns(&root_manifest_contents) {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            let directory = root.join(prefix);
            if !directory.is_dir() {
                continue;
            }

            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                push_workspace_package(&entry.path().join("Cargo.toml"), &mut packages)?;
            }
        } else {
            push_workspace_package(&root.join(pattern).join("Cargo.toml"), &mut packages)?;
        }
    }

    Ok(packages)
}

fn push_workspace_package(
    manifest_path: &Path,
    packages: &mut Vec<(String, PathBuf)>,
) -> Result<()> {
    if !manifest_path.exists() {
        return Ok(());
    }

    let contents = read_to_string(manifest_path)?;
    if toml_bool_field(&contents, "package", "publish").is_some_and(|publish| !publish) {
        return Ok(());
    }

    let name = toml_string_field(&contents, "package", "name").ok_or_else(|| {
        io::Error::other(format!(
            "workspace package is missing package.name: {}",
            manifest_path.display()
        ))
    })?;
    packages.push((name, manifest_path.to_path_buf()));
    Ok(())
}

fn workspace_member_patterns(contents: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    let mut in_workspace = false;
    let mut in_members = false;

    for line in contents.lines() {
        let stripped = line.split('#').next().unwrap_or_default().trim();
        if stripped.starts_with('[') && stripped.ends_with(']') {
            in_workspace = stripped == "[workspace]";
            in_members = false;
            continue;
        }

        if !in_workspace {
            continue;
        }

        if in_members {
            patterns.extend(quoted_strings(stripped));
            if stripped.contains(']') {
                in_members = false;
            }
            continue;
        }

        if let Some((key, value)) = stripped.split_once('=') {
            if key.trim() == "members" && value.contains('[') {
                in_members = true;
                patterns.extend(quoted_strings(value));
                if value.contains(']') {
                    in_members = false;
                }
            }
        }
    }

    patterns
}

fn quoted_strings(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = line;

    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        values.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }

    values
}

fn check_package_version(
    package_name: &str,
    contents: &str,
    workspace_version: &str,
) -> Result<()> {
    if toml_bool_field(contents, "package", "version.workspace").unwrap_or(false) {
        return Ok(());
    }

    let Some(version) = toml_string_field(contents, "package", "version") else {
        return fail(format!(
            "{package_name} is missing package version metadata"
        ));
    };

    if version == workspace_version {
        Ok(())
    } else {
        fail(format!(
            "{package_name} version {version} does not match workspace version {workspace_version}"
        ))
    }
}

fn check_package_description(
    package_name: &str,
    contents: &str,
    root_manifest_contents: &str,
) -> Result<()> {
    if toml_bool_field(contents, "package", "description.workspace").unwrap_or(false) {
        let workspace_description =
            toml_string_field(root_manifest_contents, "workspace.package", "description")
                .unwrap_or_default();
        if !workspace_description.is_empty() {
            return Ok(());
        }
    }

    if toml_string_field(contents, "package", "description").is_some_and(|value| !value.is_empty())
    {
        Ok(())
    } else {
        fail(format!("{package_name} is missing a package description"))
    }
}

fn check_package_readme(
    package_name: &str,
    contents: &str,
    manifest_dir: &Path,
    root_manifest_contents: &str,
) -> Result<()> {
    let readme = if toml_bool_field(contents, "package", "readme.workspace").unwrap_or(false) {
        workspace_package_field_from_contents(root_manifest_contents, "readme")?
    } else {
        toml_string_field(contents, "package", "readme").ok_or_else(|| {
            io::Error::other(format!("{package_name} is missing package readme metadata"))
        })?
    };

    let readme_path = if toml_bool_field(contents, "package", "readme.workspace").unwrap_or(false) {
        repo_root().join(&readme)
    } else {
        manifest_dir.join(&readme)
    };

    if readme_path.exists() {
        Ok(())
    } else {
        fail(format!(
            "{package_name} readme path does not exist: {}",
            readme_path.display()
        ))
    }
}

fn workspace_package_field_from_contents(contents: &str, field: &str) -> Result<String> {
    toml_string_field(contents, "workspace.package", field).ok_or_else(|| {
        io::Error::other(format!("workspace package metadata is missing {field}")).into()
    })
}

fn check_package_dependency_versions(
    package_name: &str,
    contents: &str,
    root_manifest_contents: &str,
    workspace_version: &str,
) -> Result<()> {
    for dependency_name in RELEASE_PACKAGES.iter().map(|(name, _)| *name) {
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if toml_bool_field(contents, section, &format!("{dependency_name}.workspace"))
                .unwrap_or(false)
            {
                let Some(requirement) =
                    workspace_dependency_version(root_manifest_contents, dependency_name)
                else {
                    return fail(format!(
                        "{package_name} dependency {dependency_name} uses workspace metadata, but [workspace.dependencies] is missing it"
                    ));
                };
                check_dependency_requirement(
                    package_name,
                    dependency_name,
                    &requirement,
                    workspace_version,
                )?;
            }

            if let Some(line) = toml_assignment_line(contents, section, dependency_name) {
                if line.contains("path") {
                    let requirement = inline_string_field(line, "version").unwrap_or_default();
                    check_dependency_requirement(
                        package_name,
                        dependency_name,
                        &requirement,
                        workspace_version,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn check_dependency_requirement(
    package_name: &str,
    dependency_name: &str,
    requirement: &str,
    workspace_version: &str,
) -> Result<()> {
    if requirement.contains(workspace_version) {
        Ok(())
    } else {
        fail(format!(
            "{package_name} dependency {dependency_name} uses version requirement {requirement:?}; expected {workspace_version}"
        ))
    }
}

fn workspace_dependency_version(contents: &str, dependency_name: &str) -> Option<String> {
    let line = toml_assignment_line(contents, "workspace.dependencies", dependency_name)?;
    inline_string_field(line, "version").or_else(|| toml_assignment_string_value(line))
}

fn toml_assignment_line<'a>(contents: &'a str, section: &'a str, key: &str) -> Option<&'a str> {
    section_lines(contents, section).find(|line| {
        line.split_once('=')
            .is_some_and(|(candidate, _)| candidate.trim() == key)
    })
}

fn toml_string_field(contents: &str, section: &str, key: &str) -> Option<String> {
    let line = toml_assignment_line(contents, section, key)?;
    toml_assignment_string_value(line)
}

fn toml_assignment_string_value(line: &str) -> Option<String> {
    let (_, value) = line.split_once('=')?;
    parse_quoted_string(value.trim())
}

fn toml_bool_field(contents: &str, section: &str, key: &str) -> Option<bool> {
    let line = toml_assignment_line(contents, section, key)?;
    let (_, value) = line.split_once('=')?;
    match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn inline_string_field(line: &str, key: &str) -> Option<String> {
    let pattern = format!("{key} = ");
    let start = line.find(&pattern)? + pattern.len();
    parse_quoted_string(line[start..].trim_start())
}

fn parse_quoted_string(value: &str) -> Option<String> {
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    Some(value[..end].to_string())
}

fn section_lines<'a>(contents: &'a str, section: &'a str) -> impl Iterator<Item = &'a str> {
    let mut active = false;
    contents.lines().filter_map(move |line| {
        let stripped = line.split('#').next().unwrap_or_default().trim();
        if stripped.starts_with('[') && stripped.ends_with(']') {
            active = stripped.trim_matches(&['[', ']'][..]) == section;
            return None;
        }

        if active && !stripped.is_empty() {
            Some(stripped)
        } else {
            None
        }
    })
}

fn command_exists(binary: &str) -> bool {
    let path = Path::new(binary);
    if path.components().count() > 1 {
        return path.is_file();
    }

    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    for directory in env::split_paths(&path_var) {
        for candidate in executable_candidates(binary) {
            if directory.join(candidate).is_file() {
                return true;
            }
        }
    }

    false
}

fn executable_candidates(binary: &str) -> Vec<OsString> {
    if cfg!(windows) && Path::new(binary).extension().is_none() {
        let pathext = env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
        env::split_paths(&pathext)
            .filter_map(|extension| extension.into_os_string().into_string().ok())
            .map(|extension| format!("{binary}{extension}").into())
            .chain(std::iter::once(binary.into()))
            .collect()
    } else {
        vec![binary.into()]
    }
}

#[cfg(test)]
mod tests {
    use super::{crc32, sha256_hex, workspace_member_patterns};

    #[test]
    fn sha256_matches_empty_string_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn crc32_matches_standard_check_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn workspace_member_patterns_read_multiline_array() {
        let manifest = r#"
[workspace]
members = [
    "engine/*",
    "tools/xtask",
]

[workspace.package]
version = "0.0.1"
"#;

        assert_eq!(
            workspace_member_patterns(manifest),
            vec!["engine/*".to_string(), "tools/xtask".to_string()]
        );
    }
}
