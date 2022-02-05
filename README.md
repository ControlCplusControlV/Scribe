
<br />
<p align="center">
    <img src="Scribe.png" alt="Logo" width="800" height="450">
  <p align="center">

 
<br />
# 📜 Scribe 📜

A minimal Yul transpiler with the target of the zk rollup Polygon Miden. The goal is to provide a foundation so people can begin testing with Miden in a languae they are familiar with, and being to start learning about developing for Miden. Since Yul is the intermediate language of Vyper, Solidity, and Yul+ we also felt it would be a good starting foundation for the transpilation efforts for those languages.

## How Does it Work?

Scribe is build using Pest for Rust, and can fully recognize Yul Grammar, we then have implemented Miden Assembly generation for a couple operations, so that fully functional Miden Assembly can be generated from Yul. Since it is a hackathon project it is still only capable of a couple of total statements (variables, for loops, lt, gt, and math) but it serves as a good foundation for future transpilation efforts nonetheless.

## How Can I Test It?

Scribe was meant to focus on real world applicability, and because of this uses Miden v0.2. Due to Miden v0.2 not being done yet certain crates of it like the zk prover are broken atm as the developers build away on the new release. So certain functionality like zk proof generation can't be done at the moment, but execution can still be tested in the zk VM environment.

First clone this repo and download its submodule

```
git clone https://github.com/ControlCplusControlV/Scribe
```

then cd into the transpiler crate

```
cd transpiler
```

Then init the git submodule, and you should be good to go!

TODO Add instructions for people to compile and run their own Yul Programs
