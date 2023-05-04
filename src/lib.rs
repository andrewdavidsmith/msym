/* MIT License
 *
 * Copyright (c) 2023 Andrew Smith
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::fs::File;
use std::io::{prelude::*,BufReader,BufWriter};
use std::path::PathBuf;
use std::os::unix::io::{FromRawFd,AsRawFd};

use msite::MSite;

fn found_symmetric(prev_cpg : &MSite, curr_cpg : &MSite) -> bool {
    // assumes check for CpG already done
    prev_cpg.strand == '+' &&
        curr_cpg.strand == '-' &&
        prev_cpg.pos + 1 == curr_cpg.pos
}

pub fn run_msym(
    input: &String,
    output: &Option<PathBuf>,
) {

    // get the input file setup
    let in_file = File::open(input).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });
    let in_file = BufReader::new(in_file);

    // get the output stream setup
    let out_writer = match output {
        Some(name) => File::create(name).unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(1);
        }),
        None => unsafe {
            File::from_raw_fd(std::io::stdout().lock().as_raw_fd())
        }
    };
    let mut out_writer = BufWriter::new(out_writer);

    // prepare for the loop
    let mut prev_site = MSite::new();
    let mut prev_is_cpg = false;

    // iterate over lines in the counts file
    for line in in_file.lines() {
        // make the current line into a site
        let the_site = MSite::build(&line.unwrap()).unwrap_or_else(|err| {
            eprintln!("failed parsing site: {err}");
            std::process::exit(1);
        });

        // only operate on CpG sites
        if the_site.is_cpg() {
            // only do anything if we have a CpG and we had one just prior
            if prev_is_cpg && found_symmetric(&prev_site, &the_site) {

                // combine both strands of the symmetric site
                prev_site.add(&the_site);

                // write the output
                writeln!(out_writer, "{}", prev_site).unwrap_or_else(|err| {
                    eprintln!("failed writing site: {err}");
                    std::process::exit(1);
                });
            }
            prev_is_cpg = true; // current site *is* CpG
        }
        else {
            prev_is_cpg = false; // current site *not* CpG
        }
        prev_site = the_site;
    }
    out_writer.flush().unwrap();
}
