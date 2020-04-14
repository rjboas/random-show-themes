# random-show-themes

Outputs random themes (songs) from user-supplied data.


## Usage

```sh
random-show-themes 10 -d dict.json -l my_list.json
```

Supply the number of themes to output.

Supply a dictionary of all known shows with `-d`. This dictionary should be a JSON file.

Each object, or `Show`, should be structured as follows:

- id (aliased to mal_id) (this is a positive integer)
- title
- (optional) url (note: URL is currently unused)
- (optional) opening_themes
- (optional) ending_themes
- (optional) other_soundtrack (aliased to soundtrack)

### Example Show from Dictionary

```json
"24833": {
        "id": 24833,
        "title": "My Show Title",
        "opening_themes": [
            "\"Seishun Satsubatsu-ron\" by 3-nen E-gumi Utatan (eps 1-6, 9-11)",
            "\"Seishun Satsubatsu-ron\" by 3-nen E-gumi Shuugakuryokou 4-han (eps 7-8)"
        ],
        "ending_themes": ["\"Hello, shooting-star\" by moumoon"],
},
```

---

Supply a list of songs to pick randomly from with `-l`. This list should be a JSON file.

It should contain a list of ids.

### Example List
```json
[24833, 30654, 28405, 9919]
```

### Options

By default it will output plain, human-readable text, one theme per line.

Results can also be output as a table using `-t` or `--table`, or as CSV using `--csv`.

Run the executable with the `--help` flag for more options.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

This is my first published project, please be kind.