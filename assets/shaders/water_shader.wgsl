// The time since startup data is in the globals binding which is part of the mesh_view_bindings import
#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

fn hash2(p: vec2<f32>) -> vec2<f32> {
   return fract(sin(vec2(dot(p, vec2(123.4, 748.6)), dot(p, vec2(547.3, 659.3))))*5232.85324);   
}
fn hash(p: vec2<f32>)  -> f32 {
  return fract(sin(dot(p, vec2(43.232, 75.876)))*4526.3257);   
}

//Based off of iq's described here: https://iquilezles.org/articles/voronoilines
fn voronoi(p: vec2<f32>)  -> f32 {
    let n: vec2<f32> = floor(p);
    let f = fract(p);
    var md = 5.0;
    var m = vec2(0.0);
    for (var i: i32 = -1;i<=1;i++) {
        for (var j: i32 = -1;j<=1;j++) {
            let g = vec2<f32>(f32(i), f32(j));
            let on = hash2(n+g);
            let o = 0.5+0.5*sin(globals.time+5.038*on);
            let r = g + o - f;
            let d = dot(r, r);
            if (d<md) {
              md = d;
              m = n+g+o;
            }
        }
    }
    return md;
}

fn ov(p0: vec2<f32>) -> f32 {
    var p = p0;
    var v = 0.0;
    var a = 0.4;
    for (var i: i32 = 0;i<3;i++) {
        v+= voronoi(p)*a;
        p*=2.0;
        a*=0.5;
    }
    return v;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let speed = 2.0;
    

    let t1_d = distance(in.uv, vec2<f32>(0.5, 0.3));
    let t2_d = distance(in.uv, vec2<f32>(0.2, 0.8));

    let t_1 = sin(t1_d * 50.0 - globals.time * speed) * 0.5 + 0.5;
    let t_2 = cos(t2_d * 10.0 - globals.time * speed) * 0.5 + 0.5;

    let brightness = t_1 + t_2 / 2.0;

    //return vec4<f32>(0.0, 0.1 + 0.1 * t_2, 0.4 + 0.2 * brightness, 1.0);

    let a = vec4<f32>(0.2, 0.4 + 0.1 * t_2, 1.0, 0.1);
    let b = vec4<f32>(0.85, 0.9, 1.0, 0.05 + 0.2 * brightness);
    return vec4<f32>(mix(a, b, smoothstep(0.0, 0.5, ov(in.uv*5.0))));
}
