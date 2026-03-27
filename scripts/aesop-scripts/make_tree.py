# if goal has a 🏁 and is leaf, should have rule pointing into it
# if goal is supposed to be proven but has no children, add empty rule
# [aesop.tree] State: 🏁 provenByRuleApplication - line comes after children info
# if state line contains 🏁 and goal has no children, add rule (rest of state line after flag emoji)
# (state line only has 🏁 for goals)

import sys
import json
from pathlib import Path

from contextlib import nullcontext

use_stdout = len(sys.argv) == 3 and sys.argv[2] == "--stdout"


def print_tree():
    # write node info to json file
    proven = "not_proven"
    if root_proven:
        proven = "proven"
    outfile = (
        "and_or_filter_unproven/"
        + proven
        + "/"
        + Path(infile).stem
        + "_"
        + str(tree_num)
        + ".json"
    )
    with open(outfile, "w") if not use_stdout else nullcontext(sys.stdout) as file:
        labels_to_goals = {}
        for goal in goals:
            labels_to_goals[goals[goal]] = goal

        file.write(
            '{ "graph": {\n  "metadata": {\n    "goal": "'
            + labels_to_goals[goals[goalgoal]]
            + '"\n  },\n  "nodes": {'
        )

        new_edges = set(edges)
        for edge in edges:
            if edge[0] in provable_rules and edge[1] not in provable_goals:
                ax = "Raxiom:" + edge[1]
                if ax not in rules:
                    rules[ax] = ax[1:]
                    new_edges.add((edge[1], ax))

        for label in labels_to_goals:
            goal = labels_to_goals[label]
            file.write(
                '\n    "'
                + goal
                + '": {\n      "label": '
                + json.dumps(label)
                + ',\n      "metadata": {\n        "kind": "OR"\n      }\n    },'
            )

        count = 1
        for rule in rules:
            file.write(
                '\n    "'
                + rule
                + '": {\n      "label": '
                + json.dumps(rules[rule])
                + ',\n      "metadata": {\n        "kind": "AND"\n      }\n    }'
            )
            if count < len(rules):
                file.write(",")
            count += 1

        file.write("\n  },")

        file.write('\n  "edges": [')

        count = 1
        for edge in new_edges:
            if edge[0].startswith("R"):
                source = edge[0]
            else:
                source = labels_to_goals[goals[edge[0]]]

            if edge[1].startswith("R"):
                target = edge[1]
            else:
                target = labels_to_goals[goals[edge[1]]]

            file.write("\n    {")
            file.write('\n      "source": "' + source + '",')
            file.write('\n      "target": "' + target + '"')
            file.write("\n    }")
            if count < len(new_edges):
                file.write(",")
            count += 1

        file.write("\n  ]")
        file.write("\n}\n}")


# map goal id to label
goals = {}
# map rule id to label
rules = {}
# [[source, target], [source, target]...]
edges = set()

id_num = -1
label = " :( "
getting_goal_label = False
leaf_goal = False
goalgoal = ""
goalgoal_found = False
extra_rapps_id = -1
note_childless_goal = False
root_proven = False
provable_goals = set()
provable_rules = set()

tree_num = 0

infile = sys.argv[1]
with open(infile, "r", encoding="utf-8") as file:
    for i, line in enumerate(file):
        line = line.replace("✅️ ", "")
        # if start of line is "info: ", write working tree to a file and start new tree
        if line[0] != " " and (len(goals) > 0 or len(rules) > 0 or len(edges) > 0):
            if len(goals) > 0 and len(rules) > 0 and len(edges) > 0:
                print_tree()

            goals = {}
            rules = {}
            edges = set()
            id_num = -1
            label = " :( "
            getting_goal_label = False
            leaf_goal = False
            goalgoal = ""
            goalgoal_found = False
            extra_rapps_id = -1
            note_childless_goal = False
            root_proven = False
            provable_rules = set()
            provable_goals = set()

            if "🏁" in line:
                root_proven = True

            tree_num += 1
        else:
            if "🏁" in line and line[0] != " ":
                root_proven = True
            # remove surrounding whitespace
            line = line.strip()

            # in pursuit of goal label?
            if getting_goal_label:
                # check if end of goal was reached
                if line[:36] == "[aesop.tree] Post-normalisation goal":
                    goals["G" + str(id_num)] = label
                    getting_goal_label = False
                    # if not goalgoal_found:
                    #     goalgoal_found = True
                    #     goalgoal = "G" + str(id_num)
                else:
                    line = line.replace("[aesop.tree]", "")
                    label = label + line + "\n"
                continue

            g = line.find("🏁 G")
            if g != -1:
                end = line.index(" ", g + 2)
                provable_goals.add(line[g + 2 : end])

            r = line.find("🏁 R")
            if r != -1:
                end = line.index(" ", r + 2)
                provable_rules.add(line[r + 2 : end])

            # goals and rules both have ID
            if line[:17] == "[aesop.tree] ID: ":
                id_num = line[17:]
                continue

            # this is a rule- get label
            if line[:19] == "[aesop.tree] Rule: ":
                label = line[19:]
                rules["R" + str(id_num)] = label
                continue

            # this is a goal- get label, which will take multiple lines
            if line[:35] == "[aesop.tree] Pre-normalisation goal":
                label = ""
                getting_goal_label = True
                continue

            if line[:26] == "[aesop.tree] Parent goal: ":
                parent = line[26:]
                if "Aesop.BuiltinRule.preprocess" in rules["R" + str(id_num)]:
                    del goals["G" + parent]
                    assert goals == {}
                    assert len(rules) == 1
                    pass
                else:
                    edges.add(("G" + parent, "R" + str(id_num)))
                continue

            if line[:33] == "[aesop.tree] Parent rapp:  some (":
                parent = line[33:-1]
                if "Aesop.BuiltinRule.preprocess" in rules["R" + parent]:
                    del rules["R" + parent]
                    assert len(goals) == 1
                    assert rules == {}
                    goalgoal = "G" + str(id_num)
                else:
                    edges.add(("R" + parent, "G" + str(id_num)))
                continue

            # if does not have child rapps, make note
            if line == "[aesop.tree] Child rapps:  []":
                note_childless_goal = True

            # if state does not contain 🏁 and does not have child rapps, add new rapp
            if line[:19] == "[aesop.tree] State:":
                if line[20] == "🏁" and note_childless_goal:
                    rules["R" + str(extra_rapps_id)] = line[21:]
                    edges.add(("G" + str(id_num), "R" + str(extra_rapps_id)))
                    extra_rapps_id -= 1

                note_childless_goal = False

if len(goals) > 0 and len(rules) > 0 and len(edges) > 0:
    print_tree()
