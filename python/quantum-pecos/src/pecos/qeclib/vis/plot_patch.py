# Copyright 2024 The PECOS Developers
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
# the License.You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.

from __future__ import annotations

from collections import defaultdict

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.patches import Circle, PathPatch
from matplotlib.path import Path

from pecos.qeclib.surface.patch_layouts.layout_4_4_4_4_rotated import gen_layout


def plot_surf_patch(
    distance: int | tuple,
    figsize=(8, 8),
    curve_height=0.4,
    curvature=0.4,
    plot_cups=True,
    plot_points=True,
):
    if isinstance(distance, int):
        width = distance
        height = distance
    else:
        width, height = distance

    node_coordinates_set1, node_coordinates_set2, polygons = gen_layout(width, height)
    return plot_two_colored(
        polygons,
        node_coordinates_set1,
        figsize=figsize,
        curve_height=curve_height,
        curvature=curvature,
        plot_cups=plot_cups,
        plot_points=plot_points,
    )


def create_cup_path(base1, base2, direction="outward", curve_height=0.5, curvature=0.5):
    """
    Create a cup-shaped path based on two base points and a specified direction.

    Parameters:
        base1 (tuple): First point of the base (x, y).
        base2 (tuple): Second point of the base (x, y).
        direction (str): 'outward' or 'inward' to indicate curve direction.
        curve_height (float): Height of the non-base point relative to the base length.
        curvature (float): Degree to which the curve broadens horizontally. Negative values invert the curvature.

    Returns:
        Path: A matplotlib path representing the cup shape.
    """
    # Calculate midpoint of the base
    mid_base = ((base1[0] + base2[0]) / 2, (base1[1] + base2[1]) / 2)

    # Calculate the length of the base
    base_length = ((base1[0] - base2[0]) ** 2 + (base1[1] - base2[1]) ** 2) ** 0.5

    # Determine the direction vector perpendicular to the base
    perpendicular_vector = (
        base2[1] - base1[1],
        base1[0] - base2[0],
    )  # Calculate a vector perpendicular to the base
    magnitude = (perpendicular_vector[0] ** 2 + perpendicular_vector[1] ** 2) ** 0.5
    perpendicular_vector = (
        perpendicular_vector[0] / magnitude,
        perpendicular_vector[1] / magnitude,
    )

    # Adjust the direction based on 'outward' or 'inward'
    if direction == "inward":
        perpendicular_vector = (-perpendicular_vector[0], -perpendicular_vector[1])

    # Calculate the non-base point (apex of the curve)
    non_base_point = (
        mid_base[0] + perpendicular_vector[0] * base_length * curve_height,
        mid_base[1] + perpendicular_vector[1] * base_length * curve_height,
    )

    # Adjust control points for curvature by pushing them horizontally
    control_point1 = (
        non_base_point[0] - (base2[0] - base1[0]) * curvature,
        non_base_point[1] - (base2[1] - base1[1]) * curvature,
    )
    control_point2 = (
        non_base_point[0] + (base2[0] - base1[0]) * curvature,
        non_base_point[1] + (base2[1] - base1[1]) * curvature,
    )

    # Create the path
    vertices = [
        base1,
        control_point1,
        non_base_point,
        control_point2,
        base2,
        base1,
    ]  # Start, curves, end, close
    codes = [
        Path.MOVETO,
        Path.CURVE3,
        Path.CURVE3,
        Path.CURVE3,
        Path.LINETO,
        Path.CLOSEPOLY,
    ]

    return Path(vertices, codes)


def build_adjacency_graph(polygons):
    """
    Build an adjacency graph for polygons that share edges.

    Parameters:
        polygons (list): List of polygons as lists of (x, y) tuples.

    Returns:
        dict: Adjacency graph.
    """
    graph = defaultdict(list)
    for i, poly1 in enumerate(polygons):
        for j, poly2 in enumerate(polygons):
            if i != j:
                edges1 = {
                    (poly1[k], poly1[(k + 1) % len(poly1)]) for k in range(len(poly1))
                }
                edges2 = {
                    (poly2[k], poly2[(k + 1) % len(poly2)]) for k in range(len(poly2))
                }
                if any(edge in edges2 or edge[::-1] in edges2 for edge in edges1):
                    graph[i].append(j)
    return graph


def bfs_two_color(graph):
    """
    Perform two-coloring using BFS.

    Parameters:
        graph (dict): Adjacency graph.

    Returns:
        dict: Color assignments for nodes.
    """
    color = {}
    for node in graph:
        if node not in color:
            # Start BFS from this node
            queue = [node]
            color[node] = 0  # Start with color 0
            while queue:
                current = queue.pop(0)
                for neighbor in graph[current]:
                    if neighbor not in color:
                        # Assign the opposite color
                        color[neighbor] = 1 - color[current]
                        queue.append(neighbor)
                    elif color[neighbor] == color[current]:
                        # Found two neighbors with the same color, graph is not two-colorable
                        msg = "Graph is not two-colorable."
                        raise ValueError(msg)
    return color


def plot_two_colored(
    polygons,
    points_to_plot,
    curve_height=0.5,
    curvature=0.5,
    plot_cups=True,
    plot_points=True,
    figsize=None,
):
    """
    Plot polygons with cups replaced for triangles and two-colored based on adjacency.

    Parameters:
        polygons (list): List of polygons as lists of (x, y) tuples.
        points_to_plot (list): List of (x, y) tuples to be plotted and labeled.
        curve_height (float): Height of the non-base point relative to the base length.
        curvature (float): Degree to which the curve broadens horizontally. Negative values invert the curvature.
    """

    if figsize is None:
        figsize = (8, 8)

    # Build the adjacency graph
    polygon_graph = build_adjacency_graph(polygons)

    # print(polygons)

    # Perform two-coloring
    polygon_colors = bfs_two_color(polygon_graph)
    # print(polygon_colors)

    # Define two colors: vibrant pastel shades of red and blue
    two_color_palette = ["#6666FF", "#FF6666"]

    # Plot setup
    fig, ax = plt.subplots(figsize=figsize)
    fig.patch.set_facecolor("#EDEDED")  # Slightly darker neutral background

    print(points_to_plot)

    # Label points_to_plot
    points_to_plot_sorted = sorted(points_to_plot, key=lambda p: (-p[1], p[0]))
    point_labels = {point: i for i, point in enumerate(points_to_plot_sorted)}

    # Determine plot scale
    x_coords, y_coords = zip(*points_to_plot)
    x_range = max(x_coords) - min(x_coords)
    y_range = max(y_coords) - min(y_coords)
    scale_factor = min(4 / x_range, 4 / y_range)  # Adjust based on plot size

    # Calculate font size based on scale factor
    radius = 0.13 + 0.05 / scale_factor
    font_size = (
        np.power(scale_factor, 0.5) * 18
    )  # Scale font size proportionally to the circle radius

    # Process the polygons
    for i, polygon in enumerate(polygons):
        if len(polygon) == 3 and plot_cups:  # For triangles, replace them with cups
            # Identify the base points and the non-base point
            if polygon[0][0] == polygon[1][0] or polygon[0][1] == polygon[1][1]:
                base1, base2, non_base = polygon[0], polygon[1], polygon[2]
            elif polygon[1][0] == polygon[2][0] or polygon[1][1] == polygon[2][1]:
                base1, base2, non_base = polygon[1], polygon[2], polygon[0]
            else:
                base1, base2, non_base = polygon[2], polygon[0], polygon[1]

            # Determine direction of the cup based on the non-base point position
            mid_base = ((base1[0] + base2[0]) / 2, (base1[1] + base2[1]) / 2)
            outward_direction = (
                "outward"
                if (non_base[0] - mid_base[0]) * (base2[1] - base1[1])
                - (non_base[1] - mid_base[1]) * (base2[0] - base1[0])
                > 0
                else "inward"
            )

            # Create and add the cup path
            cup_path = create_cup_path(
                base1,
                base2,
                direction=outward_direction,
                curve_height=curve_height,
                curvature=curvature,
            )
            cup_patch = PathPatch(
                cup_path,
                facecolor=two_color_palette[polygon_colors[i]],
                edgecolor="black",
                lw=1.5,
                alpha=0.85,
            )
            ax.add_patch(cup_patch)

        else:
            # For other polygons, draw them normally
            poly_patch = plt.Polygon(
                polygon,
                closed=True,
                facecolor=two_color_palette[polygon_colors[i]],
                edgecolor="black",
                lw=1.5,
                alpha=0.85,
            )
            ax.add_patch(poly_patch)

    if plot_points:
        # Plot numbered points as circles with labels
        for point, label in point_labels.items():
            circle = Circle(
                point,
                radius=radius,
                edgecolor="black",
                facecolor="white",
                lw=1.5,
                zorder=3,
            )
            ax.add_patch(circle)
            ax.text(
                point[0],
                point[1],
                str(label),
                color="black",
                fontsize=font_size,
                ha="center",
                va="center",
                zorder=4,
            )

    # Remove the axes
    ax.axis("off")

    # Ensure equal aspect ratio
    plt.axis("equal")
    # plt.gca().invert_yaxis()

    # plt.title("Two-Colored Cups Based on Non-Base Point Position", pad=20, color="black", fontsize=14)
    plt.tight_layout()
    # plt.show()

    return plt
