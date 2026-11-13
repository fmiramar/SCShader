// Independent scalar/corner implementation of simplex mathematics.
// Behavioral constants: pinned Hydra; algorithm background: Gustavson,
// "Simplex noise demystified" and Ashima/Gustavson webgl-noise (MIT).
// No upstream shader function body is incorporated here.
fn h_mod(x:f32, period:f32)->f32 { return x-period*floor(x/period); }
fn h_permute(x:f32)->f32 { return h_mod(x*(34.0*x+1.0),289.0); }
fn h_simplex3(point:vec3<f32>)->f32 {
    let skew=(point.x+point.y+point.z)/3.0;
    let cell=floor(point+vec3<f32>(skew));
    let local=point-cell+vec3<f32>((cell.x+cell.y+cell.z)/6.0);
    let rank=vec3<f32>(select(0.0,1.0,local.x>=local.y),select(0.0,1.0,local.y>=local.z),select(0.0,1.0,local.z>=local.x));
    let inverse=vec3<f32>(1.0)-rank;
    let corners=array<vec3<f32>,4>(vec3<f32>(0.0),min(rank,inverse.zxy),max(rank,inverse.zxy),vec3<f32>(1.0));
    let lattice=cell-289.0*floor(cell/289.0);
    var total=0.0;
    for(var corner=0u;corner<4u;corner++) {
        let offset=corners[corner];
        let displacement=local-offset+vec3<f32>(f32(corner)/6.0);
        let hash=h_permute(h_permute(h_permute(lattice.z+offset.z)+lattice.y+offset.y)+lattice.x+offset.x);
        let index=hash-49.0*floor(hash*(1.0/7.0)*(1.0/7.0));
        let gx=floor(index/7.0);let gy=floor(index-7.0*gx);
        var gradient=vec3<f32>(gx*(2.0/7.0)+(0.5/7.0-1.0),gy*(2.0/7.0)+(0.5/7.0-1.0),0.0);
        gradient.z=1.0-abs(gradient.x)-abs(gradient.y);
        if(gradient.z<=0.0) { gradient.x-=2.0*floor(gradient.x)+1.0;gradient.y-=2.0*floor(gradient.y)+1.0; }
        gradient*=1.79284291400159-0.85373472095314*dot(gradient,gradient);
        let attenuation=max(0.6-dot(displacement,displacement),0.0);
        let squared=attenuation*attenuation;
        total+=squared*squared*dot(gradient,displacement);
    }
    return total*42.0;
}
fn h_voronoi(uv:vec2<f32>, scale:f32, speed:f32, blend:f32, time:f32)->f32 {
    let position=uv*scale;let cell=floor(position);let fraction=fract(position);
    var distance=10.0;var closest=vec2<f32>(0.0);
    for(var y=-1;y<=1;y++) { for(var x=-1;x<=1;x++) {
        let neighbor=vec2<f32>(f32(x),f32(y));let address=cell+neighbor;
        let seed=fract(sin(vec2<f32>(dot(address,vec2<f32>(127.1,311.7)),dot(address,vec2<f32>(269.5,183.3))))*43758.5453);
        let center=vec2<f32>(0.5)+0.5*sin(vec2<f32>(time*speed)+6.2831*seed);
        let candidate=length(neighbor+center-fraction);
        if(candidate<distance) { distance=candidate;closest=center; }
    }}
    return dot(closest,vec2<f32>(0.3,0.6))*(1.0-blend*distance);
}
fn h_rgb_hsv(rgb:vec3<f32>)->vec3<f32> {
    let high=max(rgb.r,max(rgb.g,rgb.b));let low=min(rgb.r,min(rgb.g,rgb.b));let delta=high-low;
    var hue=0.0;
    if(rgb.r>=rgb.g && rgb.r>=rgb.b) {
        hue=abs(select(-1.0,0.0,rgb.g>=rgb.b)+abs(rgb.g-rgb.b)/(6.0*delta+1e-10));
    } else if(rgb.g>=rgb.b) { hue=abs(-1.0/3.0+(rgb.r-rgb.b)/(6.0*delta+1e-10)); }
    else { hue=abs(2.0/3.0+(rgb.r-rgb.g)/(6.0*delta+1e-10)); }
    return vec3<f32>(hue,delta/(high+1e-10),high);
}
fn h_hsv_rgb(hsv:vec3<f32>)->vec3<f32> {
    let phase=fract(hsv.x)*6.0;let chroma=hsv.y*hsv.z;
    let secondary=chroma*(1.0-abs(h_mod(phase,2.0)-1.0));
    var rgb=vec3<f32>(0.0);
    if(phase<1.0) { rgb=vec3<f32>(chroma,secondary,0.0); }
    else if(phase<2.0) { rgb=vec3<f32>(secondary,chroma,0.0); }
    else if(phase<3.0) { rgb=vec3<f32>(0.0,chroma,secondary); }
    else if(phase<4.0) { rgb=vec3<f32>(0.0,secondary,chroma); }
    else if(phase<5.0) { rgb=vec3<f32>(secondary,0.0,chroma); }
    else { rgb=vec3<f32>(chroma,0.0,secondary); }
    return rgb+vec3<f32>(hsv.z-chroma);
}
fn h_luminance(rgb:vec3<f32>)->f32 {return dot(rgb,vec3<f32>(0.2125,0.7154,0.0721));}
