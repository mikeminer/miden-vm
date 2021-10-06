use distaff::{self, StarkField, StarkProof};
use examples::{Example, ExampleOptions, ExampleType};
use log::debug;
use std::{io::Write, time::Instant};
use structopt::StructOpt;
use vm_core::utils::ToElements;

fn main() {
    // configure logging
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug)
        .init();

    // read command-line args
    let options = ExampleOptions::from_args();

    debug!("============================================================");

    let proof_options = options.get_proof_options();

    // instantiate and prepare the example
    let example = match options.example {
        ExampleType::Fib { sequence_length } => examples::fibonacci::get_example(sequence_length),
        ExampleType::Collatz { start_value } => examples::collatz::get_example(start_value),
        ExampleType::Comparison { value } => examples::comparison::get_example(value),
        ExampleType::Conditional { value } => examples::conditional::get_example(value),
        ExampleType::Merkle { tree_depth } => examples::merkle::get_example(tree_depth),
        ExampleType::Range { num_values } => examples::range::get_example(num_values),
    };

    let Example {
        program,
        inputs,
        num_outputs,
        expected_result,
    } = example;
    debug!("--------------------------------");

    // execute the program and generate the proof of execution
    let now = Instant::now();
    let (outputs, proof) =
        distaff::execute(&program, &inputs, num_outputs, &proof_options).unwrap();
    debug!("--------------------------------");
    debug!(
        "Executed program with hash {} in {} ms",
        hex::encode(program.hash()),
        now.elapsed().as_millis()
    );
    debug!("Program output: {:?}", outputs);
    assert_eq!(
        expected_result,
        outputs.to_elements(),
        "Program result was computed incorrectly"
    );

    // serialize the proof to see how big it is
    let proof_bytes = proof.to_bytes();
    debug!("Execution proof size: {} KB", proof_bytes.len() / 1024);
    debug!(
        "Execution proof security: {} bits",
        proof.security_level(true)
    );
    debug!("--------------------------------");

    // verify that executing a program with a given hash and given inputs
    // results in the expected output
    let proof = StarkProof::from_bytes(&proof_bytes).unwrap();
    let pub_inputs = inputs
        .get_public_inputs()
        .iter()
        .map(|&v| v.as_int())
        .collect::<Vec<_>>();
    let now = Instant::now();
    match distaff::verify(*program.hash(), &pub_inputs, &outputs, proof) {
        Ok(_) => println!("Execution verified in {} ms", now.elapsed().as_millis()),
        Err(msg) => println!("Failed to verify execution: {}", msg),
    }
}
