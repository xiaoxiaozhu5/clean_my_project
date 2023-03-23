use colour::{green_ln, red_ln};
use futures::{StreamExt};
use std::error::Error;
use std::path::{PathBuf};
use std::fs;
use ignore::{WalkBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

/*
* 1.find all user defined must keep file
* 2.find all folders must delete
* 3.delete all folders in step 2 except folders have file in step 1
* 4.delete all files 
*/

const IGNORE_FILE: &'static str = ".gitignore";

fn get_gitignore(dir: &str) -> Gitignore {
    let mut builder = GitignoreBuilder::new(dir);
    let error = builder.add(IGNORE_FILE);
    assert!(error.is_none(), "failed to open gitignore file");
    return builder.build().unwrap();
}

async fn solution_dir(dir: &str, preview: bool) -> Result<(), Box<dyn Error>> {
    if !PathBuf::from(dir).is_dir() {
        return Err("Invalid solution Directory".into());
    }

    let gitignore = get_gitignore(dir);
    /* 
    for result in WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(false)
        .build() {
        match result {
            Ok(entry) => {
                if entry.path().is_dir() {
                    let m = gitignore.matched_path_or_any_parents(entry.path(), true);
                    if m.is_ignore() {
                        println!("dir path: {} ignored", entry.path().display());
                        async_fs::remove_dir_all(entry.path()).await?;
                    }
                } else {
                    let m = gitignore.matched_path_or_any_parents(entry.path(), false);
                    if m.is_ignore() {
                        println!("file path: {} ignored", entry.path().display());
                        async_fs::remove_file(entry.path()).await?;
                    }
                }
            },
            Err(err) => {
                red_ln!("Error: {}", err)
            }
        }
    }
    */
    
    let (send, recv) = futures::channel::mpsc::unbounded();
    WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(false)
        .build_parallel()
        .run(move || {
                let tx = send.clone();
                let gitignore = gitignore.clone();
                Box::new(move |results| {
                    if let Ok(entry) = results {
                        if entry.path().is_dir() {
                            let m = gitignore.matched_path_or_any_parents(entry.path(), true);
                            if m.is_ignore() {
                                tx.unbounded_send(entry.path().to_owned()).unwrap();
                                //async_fs::remove_dir_all(entry.path()).await?;
                            }
                        } else {
                            let m = gitignore.matched_path_or_any_parents(entry.path(), false);
                            if m.is_ignore() {
                                tx.unbounded_send(entry.path().to_owned()).unwrap();
                                //async_fs::remove_file(entry.path()).await?;
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            }
        );
        
    let mut count :i32 = 0;
    recv.collect::<Vec<PathBuf>>()
        .await
        .into_iter()
        .for_each(|path| { 
            println!("{} will be remove", path.display());
            count+=1;
            if path.is_dir() {
                if !preview {
                    fs::remove_dir_all(path).unwrap();
                }
            } else {
                if !preview {
                    fs::remove_file(path).unwrap();
                }
            }
        });
    if !preview {
        green_ln!("{} files/folders removed", count);
    }

    Ok(())
}

fn usage() {
    println!("clean_my_project: clean up visual studio solution baesd on .gitignore file");
    println!("Usage: ./clean_my_project [option] solution_dir");
    println!("option:");
    println!("-p | --preview: show list to be delete(not really delete)");
}

fn main() -> Result<(), Box<dyn Error>> {
    async_io::block_on(async {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            usage();
        } else if args.len() == 2 {
            solution_dir(&args[1], false).await?;
        } else if args.len() == 3 {
            let project_dir = &args[2];
            let mut args = args.iter().skip(1);
            while let Some(arg) = args.next() {
                match &arg[..] {
                    "-p" | "--preview" => {
                       solution_dir(&project_dir, true).await?;
                    }
                    _ => {
                        if arg.starts_with('-') {
                            red_ln!("unknow arg: {}", arg);
                        }
                    }
                }
            }
        }
        Ok(())
    })
}
