( cd tester_for_pest && find ../noir_test_data/ -type f -name "*.nr" -exec cargo run -- "{}" \; >result.txt 2>&1 )
