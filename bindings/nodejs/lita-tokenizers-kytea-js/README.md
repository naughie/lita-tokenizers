# lita-tokenizers-kytea-js

KyTea bindings for Node.js.


## Usage

Install via npm:

```
$ npm install lita-tokenizers-kytea-js
```

Then you can use `KyTea` for performing the morphological analysis.

```javascript
import { KyTea, StringStream } from "lita-tokenizers-kytea-js";

const main = () => {
  const model = new KyTea();
  model.readModel("/path/to/model.bin");

  const input = new StringStream("すもももももももものうち");
  const output = new StringStream();

  model.tokenize(input, output);
  console.log(output.asStr());
};

main();
```

Both the input and the output are allowed to be files:

```javascript
import { FileStream, KyTea } from "lita-tokenizers-kytea-js";

const main = () => {
  const model = new KyTea();
  model.readModel("/path/to/model.bin");

  # it accepts three modes:
  # "r" (read-only), "w" (write-only, truncate), "a" (write-only, append)
  const input = new FileStream("./input.txt", "r");
  const output = new FileStream("./output.txt", "w");

  model.tokenize(input, output);
  output.flush();
};

main();
```

Also, the model itself can be loaded before calling `readModel()`:

```javascript
import { readFileSync } from "node:fs"
import { KyTea, StringStream } from "lita-tokenizers-kytea-js";

const main = () => {
  const model = new KyTea();
  # "b": binary model
  # "t": text model
  model.readModelFromBytes(readFileSync("/path/to/model.bin"), "b");

  const input = new StringStream("すもももももももものうち");
  const output = new StringStream();

  model.tokenize(input, output);
  console.log(output.asStr());
};

main();
```
