
### A bare-bones ECS proof-of-concept.  
Inspired by [`hecs`](https://github.com/Ralith/hecs), although it probably has no resemblance to it, as I just skimmed over it.  

It's goal is minimal dependency, and just "good-enough" performance for some wasm shenanigans.  
So multi-threading is not a primary goal, but it wouldn't hurt to make it correct.  
Optimizations have not been attempted yet, I'm just happy it kinda works.  

```
$ cargo run --example sandbox
```

Feel free to bash it; it's a learning project afterall.
