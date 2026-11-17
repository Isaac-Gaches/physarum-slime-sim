@group(0) @binding(0)
var trail_tex_in: texture_2d<f32>;

@group(0) @binding(1)
var trail_tex_out: texture_storage_2d<rgba16float, write>;

struct Agent {
    pos: vec2<f32>,
    dir: f32,
}

@group(0) @binding(2)
var<storage, read_write> agents: array<Agent>;

const PI: f32 = 3.14159265;

// Sensor settings
const SENSOR_DISTANCE: f32 = 12.0;
const SENSOR_ANGLE: f32 = 0.8;
const TURN_SPEED: f32 = 0.0;
const RANDOM_STEER_MIN: f32 = -0.02;
const RANDOM_STEER_MAX: f32 = 0.05;
const MOVE_SPEED: f32 = 0.1;

fn sense(agent: Agent, angle_offset: f32) -> f32 {
    let angle = agent.dir + angle_offset;
    let sensor_pos = agent.pos + vec2<f32>(cos(angle),sin(angle)) * SENSOR_DISTANCE;

    let size = textureDimensions(trail_tex_in);
    let x = clamp(i32(sensor_pos.x),0,i32(size.x) - 1);
    let y = clamp(i32(size.y) - 1 - i32(sensor_pos.y),0,i32(size.y) - 1);

    let colour = textureLoad(trail_tex_in,vec2<i32>(x, y),0);
    // Use red channel as brightness
    return colour.r;
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.x;
    if (index >= arrayLength(&agents)) {
        return;
    }

    var agent = agents[index];

    let size = textureDimensions(trail_tex_out);

    let max_x = f32(size.x - 1u);
    let max_y = f32(size.y - 1u);

    // Three sensors
    let forward = sense(agent, 0.0);
    let left = sense(agent, SENSOR_ANGLE);
    let right = sense(agent, -SENSOR_ANGLE);

    // Steer towards the brightest sensor
    let rand = hash(agent.pos.x,agent.pos.y,f32(index));
    let random_turn = RANDOM_STEER_MIN + rand * (RANDOM_STEER_MAX - RANDOM_STEER_MIN);
    if (forward >= left && forward >= right) {
        // Continue straight
    } else if (left > right) {
        agent.dir += TURN_SPEED + random_turn;
    } else if (right > left) {
        agent.dir -= TURN_SPEED + random_turn;
    }

    var dx = cos(agent.dir) * MOVE_SPEED;
    var dy = sin(agent.dir) * MOVE_SPEED;

    // Bounce off borders
    let random_bounce = ((hash(agent.pos.x,agent.pos.y,f32(index))*2.0)-1.0);
    if (agent.pos.x + dx < 0.0 || agent.pos.x + dx > max_x) {
        agent.dir = PI - agent.dir + random_bounce;
        dx = -dx;
    }
    if (agent.pos.y + dy < 0.0 || agent.pos.y + dy > max_y) {
        agent.dir = -agent.dir + random_bounce;
        dy = -dy;
    }

    agent.pos.x = clamp(agent.pos.x + dx, 0.0, max_x);
    agent.pos.y = clamp(agent.pos.y + dy, 0.0, max_y);

    agents[index] = agent;

    let px = i32(agent.pos.x);
    let py = i32(size.y) - 1 - i32(agent.pos.y);

    textureStore(
        trail_tex_out,
        vec2<i32>(px, py),
        vec4<f32>(1.0, 1.0,1.0, 1.0)
    );
}

fn hash(x: f32, y: f32, z: f32) -> f32 {
    let p = vec3<u32>(
        bitcast<u32>(x),
        bitcast<u32>(y),
        bitcast<u32>(z)
    );

    var h = p.x * 0x8da6b343u;
    h ^= p.y * 0xd8163841u;
    h ^= p.z * 0xcb1ab31fu;

    h ^= h >> 16u;
    h *= 0x7feb352du;
    h ^= h >> 15u;
    h *= 0x846ca68bu;
    h ^= h >> 16u;

    return f32(h) / 4294967296.0;
}