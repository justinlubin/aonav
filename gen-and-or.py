import sys
import random

def main():
    num_or_nodes_final = int(sys.argv[1])
    max_children_per_node = int(sys.argv[2])
    # this is number of times two nodes will be joined- number of nodes included in joins is thisi * 2
    # keep in mind need to make up for any lost 'OR' nodes
    #max_node_joins = int(sys.argv[3])
    nodes_that_need_children = [0]
    next_id = 1
    parent_to_children = {}
    num__or_nodes_so_far = 1
    ands = set()
    ors = set()
    ors.add(0)
    childless_ands = []
    while num__or_nodes_so_far < num_or_nodes_final:
        pop_node = random.randint(0, len(nodes_that_need_children) - 1)
        focus_node = nodes_that_need_children.pop(pop_node)
        if len(nodes_that_need_children) == 0:
            # must add children
            num_children = random.randint(1, max_children_per_node)
        else:
            num_children = random.randint(0, max_children_per_node)
        
        focus_children = set()
        if focus_node in ands and num_children == 0:
            childless_ands.append(focus_node)
        for i in range(0, num_children):
            nodes_that_need_children.append(next_id)
            focus_children.add(next_id)
            if focus_node in ors:
                ands.add(next_id)
            else:
                ors.add(next_id)
                num__or_nodes_so_far += 1
            next_id += 1
            if num__or_nodes_so_far == num_or_nodes_final:
                break;
        
        parent_to_children[focus_node] = focus_children
    # now remove all leaf 'AND' nodes
    for n in nodes_that_need_children + childless_ands:
        if n in ands:
            ands.remove(n)
            remove_from_parent_to_children = set()
            for parent in parent_to_children:
                pc = parent_to_children[parent]
                if n in pc:
                    pc.remove(n)
                    if len(pc) == 0:
                        remove_from_parent_to_children.add(parent)
                    else:
                        parent_to_children[parent] = pc
            for r in remove_from_parent_to_children:
                parent_to_children.pop(r)

    '''
    # join some nodes to make it a graph
    num_join = random.randint(0, max_node_joins)
    for j in range (0, num_join):
        node1 = random.choice(list(ors))
        node2 = node1
        while node2  == node1:
            node2 = random.choice(list(ors))

        # node1 should be higher in tree or equal level to node2? yes if node1 is root 0
        if node2 > node1:
            temp = node1
            node1 = node2
            node2 = temp
        
        ors.remove(node2)
        

        # give all of node2's children to node1
        if node2 in parent_to_children:
            node1_children = set()
            if node1 in parent_to_children:
                node1_children = parent_to_children[node1]
            parent_to_children[node1] = node1_children.union(parent_to_children[node2])
            parent_to_children.pop(node2)

        # wherever node2 appears as a child, replace with node1
        
        for parent in parent_to_children:
            children = parent_to_children[parent]
            if node2 in children:
                children.remove(node2)
                children.add(node1)
        '''
                
    
    print("{ \"graph\": {\n  \"metadata\": {\n    \"goal\": \"0\"\n  },\n  \"nodes\": {")

    # now print all and nodes
    for node_id in ands:
        print("    \"" + str(node_id) + "\": {\n      \"metadata\": {\n        \"kind\": \"AND\"\n      }\n    },")

    # print all or nodes
    o = 1
    for node_id in ors:
        # don't put comma at end
        comma = ","
        if o == len(ors):
            comma = ""
        print("    \"" + str(node_id) + "\": {\n      \"metadata\": {\n        \"kind\": \"OR\"\n      }\n    }" + comma)
        o += 1

    print("  },\n  \"edges\": [")
    

    # print all edges
    p = 1
    for parent in parent_to_children:
        c = 1
        for child in parent_to_children[parent]:
            comma = ","
            if p == len(parent_to_children) and c == len(parent_to_children[parent]):
                comma = ""
            print("    {\n      \"source\": \"" + str(parent) + "\",\n      \"target\": \"" + str(child) + "\"\n    }" + comma)
            c += 1
        p += 1

    print("  ]\n} }")


if __name__ == "__main__":
    main()
