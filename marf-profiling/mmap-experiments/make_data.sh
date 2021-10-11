head -n 1000000 ~/data/no_mmap.log | python3 extract_data.py no_mmap > data/no_mmap.txt
head -n 1000000 ~/data/with_mmap.log | python3 extract_data.py with_mmap > data/with_mmap.txt
