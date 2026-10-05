#version 330
out vec4 finalColor;
void main()
{
    float d = gl_FragCoord.z; // window-space depth
    finalColor = vec4(d, d, d, 1.0);
}
