# Copyright 2018 The PECOS Developers
# Copyright 2018 National Technology & Engineering Solutions of Sandia, LLC (NTESS). Under the terms of Contract
# DE-NA0003525 with NTESS, the U.S. Government retains certain rights in this software.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
# the License.You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.


def calc_id2pos(i, width, height):
    # return (1+i*2)%(dz*2), (dx-(i//dz))*2-1
    return (1 + i * 2) % (width * 2), (height - (i // width)) * 2 - 1


def calc_pos2id(x, y, width, height):
    # return (x-1)//2+((2*dx-y-1)//2)*dz
    return (x - 1) // 2 + ((2 * height - y - 1) // 2) * width


def get_stab_gens(height: int, width: int):
    """Generate rectangular rotated surface code patch layout for a 4.4.4.4 lattice."""

    lattice_height = height * 2
    lattice_width = width * 2

    polygons_0 = []
    polygons_1 = []

    for x in range(lattice_width + 1):
        for y in range(lattice_height + 1):
            if 0 < x < lattice_width and 0 < y < lattice_height:
                # Interior

                if x % 2 == 1 and y % 2 == 1:  # That is, both coordinates are odd...
                    pass

                elif x % 2 == 0 and y % 2 == 0:
                    # Bulk checks
                    poly = [
                        calc_pos2id(x - 1, y + 1, width, height),
                        calc_pos2id(x + 1, y + 1, width, height),
                        calc_pos2id(x - 1, y - 1, width, height),
                        calc_pos2id(x + 1, y - 1, width, height),
                    ]

                    if ((x + y) / 2) % 2 == 0:
                        polygons_0.append(poly)
                    else:
                        polygons_1.append(poly)

            elif 0 < x < lattice_width or 0 < y < lattice_height:
                # Not the corners or the interior

                if y == 0:
                    # Bottom: X checks

                    if x != 0 and x % 4 == 0:
                        poly = [
                            calc_pos2id(x - 1, y + 1, width, height),
                            calc_pos2id(x + 1, y + 1, width, height),
                        ]
                        polygons_0.append(poly)

                elif x == 0:
                    # Left: Z checks

                    if (y - 2) % 4 == 0:
                        poly = [
                            calc_pos2id(x + 1, y + 1, width, height),
                            calc_pos2id(x + 1, y - 1, width, height),
                        ]
                        polygons_1.append(poly)

                if y == lattice_height:
                    # Top: X checks

                    if height % 2 == 0:
                        if x != 0 and x % 4 == 0:
                            poly = [
                                calc_pos2id(x - 1, y - 1, width, height),
                                calc_pos2id(x + 1, y - 1, width, height),
                            ]
                            polygons_0.append(poly)

                    else:
                        if (x - 2) % 4 == 0:
                            poly = [
                                calc_pos2id(x - 1, y - 1, width, height),
                                calc_pos2id(x + 1, y - 1, width, height),
                            ]
                            polygons_0.append(poly)

                elif x == lattice_width:
                    # Right: Z checks

                    if width % 2 == 1:
                        if y != 0 and y % 4 == 0:
                            poly = [
                                calc_pos2id(x - 1, y + 1, width, height),
                                calc_pos2id(x - 1, y - 1, width, height),
                            ]
                            polygons_1.append(poly)
                    else:
                        if (y - 2) % 4 == 0:
                            poly = [
                                calc_pos2id(x - 1, y + 1, width, height),
                                calc_pos2id(x - 1, y - 1, width, height),
                            ]
                            polygons_1.append(poly)

    stab_gens = []

    for poly in polygons_0:
        stab_gens.append(("X", tuple(poly)))
    for poly in polygons_1:
        stab_gens.append(("Z", tuple(poly)))

    return stab_gens


def order_coords_counter_clockwise(coords):
    """
    Reorders a list of coordinates in approximate counter-clockwise order using x, y sorting.

    Parameters:
        coords (list): List of (x, y) tuples.

    Returns:
        list: List of (x, y) tuples ordered counter-clockwise.
    """
    if len(coords) < 3:
        return coords  # No reordering needed for lines or single points

    # Calculate centroid
    cx = sum(x for x, y in coords) / len(coords)
    cy = sum(y for x, y in coords) / len(coords)

    # Sort based on quadrant and relative position
    def sort_key(point):
        x, y = point
        if x >= cx and y >= cy:  # Top-right
            return 0, x
        elif x < cx and y >= cy:  # Top-left
            return 1, -y
        elif x < cx and y < cy:  # Bottom-left
            return 2, -x
        else:  # Bottom-right
            return 3, y

    return sorted(coords, key=sort_key)


def get_plot_info(dz, dx, stab_gens):
    polygon_colors = {}
    for i, (pauli, _) in enumerate(stab_gens):
        polygon_colors[i] = 0 if pauli == "X" else 1

    polygons = []
    for _, datas in stab_gens:
        temp = []
        for id_ in datas:
            temp.append(calc_id2pos(id_, dz, dx))

        polygons.append(temp)

    polygons = [order_coords_counter_clockwise(coords) for coords in polygons]

    for coords in polygons:
        # make a triangle
        if len(coords) == 2:
            # Work out the original (x, y) of the dual node
            (x1, y1), (x2, y2) = coords
            if y1 == y2 == 1:
                coords.insert(0, (x1 + 1, 0))
            elif y1 == y2 == 2 * dx - 1:
                coords.insert(0, (x1 + 1, y1 + 1))
            elif x1 == x2 == 1:
                coords.insert(0, (x1 - 1, y1 - 1))
            elif x1 == x2 == 2 * dz - 1:
                coords.insert(0, (x1 + 1, y1 + 1))
            else:
                msg = f"Unexpected digon coordinates: {coords}"
                raise Exception(msg)

    nodes = [calc_id2pos(i, dz, dx) for i in range(dx * dz)]

    return polygons, polygon_colors, nodes


def gen_layout(width: int, height: int):
    """Generate rectangular rotated surface code patch layout for a 4.4.4.4 lattice."""
    lattice_height = height * 2
    lattice_width = width * 2

    nodes = []
    dual_nodes = []
    polygons_0 = []
    polygons_1 = []

    for x in range(lattice_width + 1):
        for y in range(lattice_height + 1):
            if 0 < x < lattice_width and 0 < y < lattice_height:
                # Interior

                if x % 2 == 1 and y % 2 == 1:  # That is, both coordinates are odd...
                    nodes.append((x, y))

                elif x % 2 == 0 and y % 2 == 0:
                    dual_nodes.append((x, y))
                    poly = [
                        (x - 1, y + 1),
                        (x - 1, y - 1),
                        (x + 1, y - 1),
                        (x + 1, y + 1),
                    ]
                    polygons_1.append(poly)

            elif 0 < x < lattice_width or 0 < y < lattice_height:
                # Not the corners or the interior

                if y == 0:
                    # Top: X checks

                    if x != 0 and x % 4 == 0:
                        dual_nodes.append((x, y))
                        poly = [(x, y), (x - 1, y + 1), (x + 1, y + 1)]
                        polygons_0.append(poly)

                elif x == 0:
                    # Left column: X checks

                    if (y - 2) % 4 == 0:
                        dual_nodes.append((x, y))
                        poly = [(x, y), (x + 1, y + 1), (x + 1, y - 1)]
                        polygons_0.append(poly)

                if y == lattice_height:
                    # Bottom: X checks

                    if height % 2 == 0:
                        if x != 0 and x % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x + 1, y - 1)]
                            polygons_0.append(poly)

                    else:
                        if (x - 2) % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x + 1, y - 1)]
                            polygons_0.append(poly)

                elif x == lattice_width:
                    # Right column: X checks

                    if width % 2 == 1:
                        if y != 0 and y % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x - 1, y + 1)]
                            polygons_0.append(poly)
                    else:
                        if (y - 2) % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x - 1, y + 1)]
                            polygons_0.append(poly)
    polygons = []
    polygons.extend(polygons_0)
    polygons.extend(polygons_1)

    return nodes, dual_nodes, polygons
