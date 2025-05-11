import pandas as pd
import matplotlib.pyplot as plt

# === Load routed_output.csv ===
df = pd.read_csv("routed_output.csv")

grid_size = 10
nets = df['net'].unique()

# === Detect vias (layer switches) ===
vias = []
for net in nets:
    path = df[df['net'] == net].reset_index(drop=True)
    for i in range(1, len(path)):
        if path.loc[i, 'layer'] != path.loc[i - 1, 'layer']:
            vias.append((path.loc[i, 'x'], path.loc[i, 'y']))

# ========== PLOT 1: Side-by-Side Layers ==========
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 6))
fig.suptitle("Maze Routing Visualization: Layer 1 vs Layer 2")

for ax, layer_num, title in zip([ax1, ax2], [1, 2], ["Layer 1 (Horizontal Preferred)", "Layer 2 (Vertical Preferred)"]):
    ax.set_title(title)
    ax.set_xlim(-0.5, grid_size - 0.5)
    ax.set_ylim(-0.5, grid_size - 0.5)
    ax.set_xticks(range(grid_size))
    ax.set_yticks(range(grid_size))
    ax.grid(True)
    ax.invert_yaxis()

    for net in nets:
        segment = df[(df['net'] == net) & (df['layer'] == layer_num)]
        if not segment.empty:
            x = segment['y']
            y = segment['x']
            ax.plot(x, y, '-o', label=net)
            ax.plot(x.iloc[0], y.iloc[0], 'go')  # Start
            ax.plot(x.iloc[-1], y.iloc[-1], 'ro')  # End

    # Show vias
    for vx, vy in vias:
        ax.plot(vy, vx, 'ks', markersize=10, label='Via' if (vx, vy) == vias[0] else "")

ax1.legend()
ax2.legend()
plt.tight_layout()
plt.show()

# ========== PLOT 2: Combined Routing View ==========
fig, ax = plt.subplots(figsize=(6, 6))
ax.set_title("Combined Routing View (Layers 1 & 2)")
ax.set_xlim(-0.5, grid_size - 0.5)
ax.set_ylim(-0.5, grid_size - 0.5)
ax.set_xticks(range(grid_size))
ax.set_yticks(range(grid_size))
ax.grid(True)
ax.invert_yaxis()

colors = {'1': 'blue', '2': 'orange'}

for net in nets:
    path = df[df['net'] == net].reset_index(drop=True)
    for i in range(1, len(path)):
        x0, y0 = path.loc[i - 1, 'x'], path.loc[i - 1, 'y']
        x1, y1 = path.loc[i, 'x'], path.loc[i, 'y']
        layer = str(path.loc[i, 'layer'])
        ax.plot([y0, y1], [x0, x1], '-o', color=colors[layer], label=f"{net} (L{layer})" if i == 1 else "")

    ax.plot(path.loc[0, 'y'], path.loc[0, 'x'], 'go')  # Start
    ax.plot(path.loc[len(path)-1, 'y'], path.loc[len(path)-1, 'x'], 'ro')  # End

for vx, vy in vias:
    ax.plot(vy, vx, 'ks', markersize=10, label='Via' if (vx, vy) == vias[0] else "")

ax.legend()
plt.tight_layout()
plt.show()
