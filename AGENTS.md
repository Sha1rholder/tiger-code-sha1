`uv run src/main.py --deploy` to update dictionaries and redeploy Weasel. Weasel log files are stored in `%TEMP%\rime.weasel\`.

`uv run src/main.py --debug` to update dictionaries and write debug files to `temp/`:
- `temp/add.tsv` — merged and sorted Chinese additional entries
- `temp/add.txt` — Chinese additional entries as plain text
- `temp/en_dict.tsv` — English base entries with ranking metrics (word, frequency, boosted_frequency, demotion_count) for reviewing sort order and derivation logic

Update `README.md` and `src\README.md` after implementing new features or modifying behaviors.

Code Style:
- Always use hard tabs for indentation and alignment.
- Do not add space between Chinese characters and English words, backticks, or numbers.
- All functions must have docstrings written in Chinese.
- Omit the Chinese period `。` at the end of paragraphs and docstrings.
