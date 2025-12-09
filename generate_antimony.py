import math

def generate_antimony_svg():
    width = 200
    height = 200
    cx = width / 2
    cy = height / 2
    
    # Electron configuration for Antimony (Sb, Z=51)
    # K=2, L=8, M=18, N=18, O=5
    shells = [2, 8, 18, 18, 5]
    
    # Animation speeds (seconds per revolution)
    speeds = [4, 8, 15, 25, 40]
    direction = ["normal", "reverse", "normal", "reverse", "normal"]
    
    # Brighter, high-contrast electric colors
    colors = ["#00ffff", "#00bfff", "#4361ee", "#7209b7", "#f72585"]
    
    svg = []
    svg.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">')
    
    # CSS Styles for Animation
    svg.append('<style>')
    svg.append('.shell { transform-origin: 100px 100px; }')
    svg.append('@keyframes rotate { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }')
    
    for i in range(len(shells)):
        anim_name = f'rotate-{i}'
        svg.append(f'.shell-{i} {{ animation: rotate {speeds[i]}s linear infinite {direction[i]}; }}')
        
    svg.append('</style>')
    
    # Nucleus (Glowing)
    svg.append('<defs>')
    svg.append('<radialGradient id="nucleus-glow" cx="50%" cy="50%" r="50%">')
    svg.append('<stop offset="0%" stop-color="#ff0080" stop-opacity="1" />')
    svg.append('<stop offset="100%" stop-color="#ff0080" stop-opacity="0" />')
    svg.append('</radialGradient>')
    svg.append('</defs>')
    
    svg.append(f'<circle cx="{cx}" cy="{cy}" r="12" fill="url(#nucleus-glow)" />')
    svg.append(f'<circle cx="{cx}" cy="{cy}" r="6" fill="#ffffff" opacity="1.0" />')
    
    
    # Shells
    max_radius = 90
    min_radius = 20
    # spread radii slightly non-linear for better visual separation
    radii = [20, 35, 50, 70, 90]
    
    for i, count in enumerate(shells):
        r = radii[i]
        
        # Group for rotation
        svg.append(f'<g class="shell shell-{i}">')
        
        # Orbit ring (Higher contrast)
        svg.append(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{colors[i]}" stroke-width="1.5" opacity="0.6" stroke-dasharray="3,3" />')
        
        # Electrons (Particles - Bright & Solid)
        for j in range(count):
            angle_offset = (i * math.pi / 4) # staggered starting angles
            angle = (2 * math.pi * j) / count + angle_offset
            ex = cx + r * math.cos(angle)
            ey = cy + r * math.sin(angle)
            
            # Electron glow
            svg.append(f'<circle cx="{ex}" cy="{ey}" r="3.5" fill="{colors[i]}" opacity="1.0" filter="drop-shadow(0 0 2px {colors[i]})" />')
            
        svg.append('</g>')
            
    svg.append('</svg>')
    return "\n".join(svg)

if __name__ == "__main__":
    with open("WELCOME/assets/islands/antimony.svg", "w") as f:
        f.write(generate_antimony_svg())
    print("Generated WELCOME/assets/islands/antimony.svg")
