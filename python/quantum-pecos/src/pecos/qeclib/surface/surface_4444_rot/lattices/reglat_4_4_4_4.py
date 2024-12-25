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
