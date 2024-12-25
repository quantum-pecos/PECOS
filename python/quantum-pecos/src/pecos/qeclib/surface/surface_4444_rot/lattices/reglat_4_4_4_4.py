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


def gen_layout(width: int, height: int):
    """Generate lattice layout for a 4.4.4.4"""
    lattice_height = height * 2
    lattice_width = width * 2

    nodes = []
    dual_nodes = []
    polygons = []

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
                    polygons.append(poly)

            elif 0 < x < lattice_width or 0 < y < lattice_height:
                # Not the corners or the interior

                if y == 0:
                    # Top: X checks

                    if x != 0 and x % 4 == 0:
                        dual_nodes.append((x, y))
                        poly = [(x, y), (x - 1, y + 1), (x + 1, y + 1)]
                        polygons.append(poly)

                elif x == 0:
                    # Left column
                    # X checks

                    if (y - 2) % 4 == 0:
                        dual_nodes.append((x, y))
                        poly = [(x, y), (x + 1, y + 1), (x + 1, y - 1)]
                        polygons.append(poly)

                if y == lattice_height:
                    # Bottom: X checks

                    if height % 2 == 0:
                        if x != 0 and x % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x + 1, y - 1)]
                            polygons.append(poly)

                    else:
                        if (x - 2) % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x + 1, y - 1)]
                            polygons.append(poly)

                elif x == lattice_width:
                    # Right column
                    # X checks

                    if width % 2 == 1:
                        if y != 0 and y % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x - 1, y + 1)]
                            polygons.append(poly)
                    else:
                        if (y - 2) % 4 == 0:
                            dual_nodes.append((x, y))
                            poly = [(x, y), (x - 1, y - 1), (x - 1, y + 1)]
                            polygons.append(poly)
    return nodes, dual_nodes, polygons
