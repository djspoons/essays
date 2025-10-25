use std::error;
use std::fs;
use std::path;
use std::process;

use clap::Parser as ClapParser;
use comrak::format_commonmark;
use comrak::nodes;
use comrak::{Arena, Options, parse_document};

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,
    #[arg(short = 'O', long, default_value = ".")]
    output_dir: String,
    #[arg(short = 'o', long, default_value = "")]
    output_file: String,
    #[arg(
        long,
        default_value = "target/release/tuun",
        help = "Path to the tuun executable, relative to tuun-dir"
    )]
    tuun_exec: String,
    #[arg(long, default_value = ".", help = "Directory in which to run tuun")]
    tuun_dir: String,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.math_dollars = true;
    options.extension.footnotes = true;

    let input = fs::read_to_string(&args.input_file)?;
    let root = parse_document(&arena, &input, &options);

    // TODO maybe if output_dir isn't set, then use the directory of the input file?

    // Iterate over all the descendants of root.
    for node in root.children() {
        let ast = node.data.borrow_mut();
        if let nodes::NodeValue::CodeBlock(ref code) = ast.value {
            if code.info.starts_with("tuun ") {
                println!("Processing tuun block ({}): {}", code.info, code.literal);
                let tmp_dir = std::env::temp_dir().join("md-tuuner");
                fs::create_dir_all(&tmp_dir)?;
                let mut command_args = vec!["--ui=false", "--date-format=", "--buffer-size=16384"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                // Add the arguments from the info string
                command_args.extend(
                    code.info
                        .trim_start_matches("tuun ")
                        .trim()
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                );
                // Remove comments and add each line as a -p argument
                command_args.extend(
                    code.literal
                        .lines()
                        .map(|line| {
                            if let Some(comment_index) = line.find("//") {
                                &line[..comment_index]
                            } else {
                                line
                            }
                        })
                        .map(|s| vec!["-p", s.trim()])
                        .flatten()
                        .map(|s| s.to_string()),
                );

                let mut command = process::Command::new(&args.tuun_exec);
                command.args(command_args);

                // Leave a comment with the command line
                let comment = nodes::NodeHtmlBlock {
                    block_type: 2, // comment
                    literal: format!(
                        "<!--\ntuun {}\n-->\n",
                        command
                            .get_args()
                            .map(|s| format!("{:?}", s))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                };
                let comment = arena.alloc(nodes::NodeValue::HtmlBlock(comment).into());
                node.insert_before(comment);

                command.arg(format!("--output-dir={}", tmp_dir.display()).as_str());
                println!("Running tuun in {:?} as: {:?}", &args.tuun_dir, command);
                command.current_dir(&args.tuun_dir);
                let output = command.output()?;
                if !output.status.success() {
                    eprintln!("tuun failed with status: {}", output.status);
                    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                    fs::remove_dir_all(&tmp_dir)?;
                    std::process::exit(1);
                }

                let mut node_list = nodes::NodeList::default();
                node_list.tight = true;
                let list = arena.alloc(nodes::NodeValue::List(node_list).into());

                let mut audio_files: Vec<_> = fs::read_dir(&tmp_dir)?.collect::<Result<_, _>>()?;
                audio_files.sort_by_key(|entry| entry.file_name());
                for audio_file in audio_files {
                    let file_name = audio_file.file_name();
                    // Copy the file to the output directory
                    let dest_path = path::Path::new(&args.output_dir).join(&file_name);
                    println!("Copying {:?} to {:?}", tmp_dir.join(&file_name), &dest_path);
                    fs::copy(tmp_dir.join(&file_name), &dest_path)?;

                    // Insert a the audio tags
                    let paragraph = arena.alloc(nodes::NodeValue::Paragraph.into());
                    let audio = arena.alloc(nodes::NodeValue::HtmlInline(format!(
                            "<audio controls>\n<source src=\"{}\" type=\"audio/wav\">\nYour browser does not support the audio element.\n</audio>",
                            dest_path.file_name().unwrap().display()
                        )).into());
                    paragraph.append(audio);

                    let item = arena.alloc(nodes::NodeValue::Item(node_list).into());
                    item.append(paragraph);
                    list.append(item);
                }
                fs::remove_dir_all(&tmp_dir)?;

                node.insert_before(list);
                node.detach();
            }
        }
    }

    let buf = &mut String::new();
    format_commonmark(root, &options, buf)?;
    let output_file;
    if args.output_file.is_empty() {
        if args.input_file.ends_with(".base.md") {
            let base_input_stem = path::Path::new(&args.input_file).file_stem().unwrap();
            let input_stem = path::Path::new(base_input_stem).file_stem().unwrap();
            output_file = path::Path::new(&args.output_dir)
                .join(input_stem)
                .with_extension("md");
        } else {
            output_file = path::Path::new(&args.output_dir)
                .join(path::Path::new(&args.input_file).file_stem().unwrap())
                .with_extension("out.md");
        }
    } else {
        output_file = path::Path::new(&args.output_dir).join(path::Path::new(&args.output_file));
    }
    fs::remove_file(&output_file)?;
    fs::write(&output_file, &buf)?;
    let metadata = fs::metadata(&output_file)?;
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&output_file, permissions)?;

    Ok(())
}
