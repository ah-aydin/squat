import os
import subprocess
import filecmp

GREEN = '\033[92m'
RED = '\033[91m'
ENDC = '\033[0m'

def run_vm(script_name):
    command = f"cargo run --release -- -f test_scripts/{script_name}.squat"
    output_file = f"test_scripts/output/{script_name}.out"

    with open(output_file, "w") as output:
        subprocess.run(command, shell=True, stdout=output, stderr=subprocess.PIPE)

def compare_output_files(script_name):
    generated_file = f"test_scripts/output/{script_name}.out"
    expected_file = f"test_scripts/expected_output/{script_name}.out"
    return filecmp.cmp(generated_file, expected_file)

def main():
    squat_files = [filename[:-6] for filename in os.listdir("test_scripts") if filename.endswith(".squat")]
    squat_files = sorted(squat_files)    

    i = 0
    success = 0
    for script_name in squat_files:
        i += 1
        run_vm(script_name)
        if compare_output_files(script_name):
            success += 1
            print(f"Test {i}/{len(squat_files)} ({script_name}) passed")
        else:
            print(f"{RED}Test {i}/{len(squat_files)} ({script_name}) failed{ENDC}")
    print(f"{success}/{len(squat_files)} passed")

if __name__ == "__main__":
    main()
