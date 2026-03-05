Parse and include GFD files (most commonly GX2 shaders) at compile time.

# Usage

```rust
use cafe_shader::include_shader;

include_shader!(MY_SHADER, "path/to/shader.gfd");

fn main() {
    use_shader(MY_SHADER.vertex[0], MY_SHADER.pixel[0]);
}
```
