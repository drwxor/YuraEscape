#version 330

in vec3 fragPosition;
in vec2 fragTexCoord;
in vec3 fragNormal;

uniform sampler2D texture0;
uniform vec4 colDiffuse;

out vec4 finalColor;

uniform vec3 lightPos;
uniform vec3 spotDir;
uniform float spotInnerCos;
uniform float spotOuterCos;
uniform vec4 lightColor;
uniform vec4 ambient;
uniform vec3 viewPos;

uniform mat4 lightVP;
uniform sampler2D shadowMap;
uniform int shadowMapResolution;

void main()
{
    vec4 texelColor = texture(texture0, fragTexCoord);
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(viewPos - fragPosition);

    vec3 L = normalize(lightPos - fragPosition);

    float theta = dot(normalize(-L), normalize(spotDir));
    float intensity = 0.0;
    if (theta > spotOuterCos)
    {
        float t = (theta - spotOuterCos) / max(0.00001, (spotInnerCos - spotOuterCos));
        intensity = clamp(t, 0.0, 1.0);
    }

    float NdotL = max(dot(N, L), 0.0);
    vec3 diffuse = lightColor.rgb * NdotL * intensity;

    float specCo = 0.0;
    if (NdotL > 0.0) specCo = pow(max(0.0, dot(V, reflect(-L, N))), 16.0);

    vec3 result = texelColor.rgb * ((colDiffuse.rgb + vec3(specCo)) * diffuse);

    vec4 fragPosLightSpace = lightVP * vec4(fragPosition, 1.0);
    fragPosLightSpace.xyz /= fragPosLightSpace.w;
    fragPosLightSpace.xyz = (fragPosLightSpace.xyz + 1.0) / 2.0;
    vec2 sampleCoords = fragPosLightSpace.xy;
    float curDepth = fragPosLightSpace.z;

    float bias = 1.0;
    int shadowCounter = 0;
    const int numSamples = 9;
    vec2 texelSize = vec2(1.0/float(shadowMapResolution));
    for (int x = -1; x <= 1; x++)
    {
        for (int y = -1; y <= 1; y++)
        {
            float sampleDepth = texture(shadowMap, sampleCoords + texelSize * vec2(x,y)).r;
            if (curDepth - bias > sampleDepth) shadowCounter++;
        }
    }
    float shadowFactor = float(shadowCounter) / float(numSamples);
    vec3 shaded = mix(result, vec3(0.0), shadowFactor);

    shaded += texelColor.rgb * (ambient.rgb/10.0) * colDiffuse.rgb;

    finalColor = vec4(pow(shaded, vec3(1.0/2.2)), texelColor.a);
}
