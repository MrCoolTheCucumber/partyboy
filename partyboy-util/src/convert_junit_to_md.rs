use std::fmt::Write;
use std::fs;

use clap::Parser;
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    path: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "testsuites")]
struct TestSuites {
    #[serde(rename = "testsuite")]
    test_suites: Vec<TestSuite>,
    tests: i32,
    failures: i32,
    errors: i32,
}

#[derive(Deserialize, Debug)]
struct TestSuite {
    #[serde(rename = "testcase")]
    testcases: Vec<TestCase>,
    name: String,
    tests: i32,
    failures: i32,
    errors: i32,
}

#[derive(Deserialize, Debug)]
struct TestCase {
    name: String,
    failure: Option<Failure>,
}

#[derive(Deserialize, Debug)]
struct Failure {}

pub fn execute(args: Args) {
    let junit = fs::read_to_string(args.path).unwrap();
    let test_suites: TestSuites = from_str(&junit).expect("Unable to deserialize xml");

    let mut markdown = String::new();
    writeln!(&mut markdown, "# Test Report").unwrap();

    writeln!(&mut markdown, "### Aggregated Results").unwrap();
    writeln!(&mut markdown, "| | Count |").unwrap();
    writeln!(&mut markdown, "| - | ---:|").unwrap();
    writeln!(&mut markdown, "| **Tests** | {} |", test_suites.tests).unwrap();
    writeln!(&mut markdown, "| **Failures** | {} |", test_suites.failures).unwrap();
    writeln!(&mut markdown, "| **Errors** | {} |\n", test_suites.errors).unwrap();

    test_suites
        .test_suites
        .iter()
        .for_each(|suite| write_test_suite(&mut markdown, suite));

    fs::write("TestReport.md", markdown).expect("Unable to create test report file");
}

fn write_test_suite(markdown: &mut String, test_suite: &TestSuite) {
    writeln!(markdown, "## {}", test_suite.name).unwrap();

    writeln!(markdown, "| | Count |").unwrap();
    writeln!(markdown, "| - | ---:|").unwrap();
    writeln!(markdown, "| **Tests** | {} |", test_suite.tests).unwrap();
    writeln!(markdown, "| **Failures** | {} |", test_suite.failures).unwrap();
    writeln!(markdown, "| **Errors** | {} |\n", test_suite.errors).unwrap();

    writeln!(markdown, "| Test | Result |").unwrap();
    writeln!(markdown, "| - | ---:|").unwrap();

    test_suite.testcases.iter().for_each(|case| {
        let result = match case.failure {
            None => "✅",
            Some(_) => "❌",
        };
        writeln!(markdown, "| {} | {} |", case.name, result).unwrap();
    });

    writeln!(markdown).unwrap();
}
