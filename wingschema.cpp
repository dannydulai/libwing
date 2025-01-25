#include <cstring>
#include <iostream>
#include <fstream>

#include "WingConsole.h"

using namespace std;
using namespace std::chrono;

static ofstream dataFile;
static ofstream schemaFile;

static map<uint32_t, vector<uint32_t>> _nodeParentToChildren;
static map<uint32_t, NodeDefinition>        _nodeIdToDef;

void
printNode(int nodeId, bool recurs)
{
    auto def = _nodeIdToDef[nodeId];

    string fullname = def.name;
    auto n = def;
    while (n.parentId != 0) {
        if (_nodeIdToDef.find(n.parentId) == _nodeIdToDef.end()) {
            fullname = "\?\?\?/" + fullname;
            break;
        } else {
            n = _nodeIdToDef[n.parentId];
            if (n.name.empty()) {
                fullname = to_string(n.index) + "/" + fullname;
            } else {
                fullname = n.name + "/" + fullname;
            }
        }
    }
    if (n.parentId == 0) fullname = "/" + fullname;

    vector<string> parts;
    parts.push_back(format("\"id\": {:10d}", def.id));
    parts.push_back(format("\"fullName\": \"{}\"", fullname));

    if (def.index != 0)        parts.push_back(format("\"idx\": \"{}\"", def.index));
    if (!def.name.empty())     parts.push_back(format("\"name\": \"{}\"", def.name));
    if (!def.longname.empty()) parts.push_back(format("\"longname\": \"{}\"", def.longname));

    switch (def.getType()) {
        case WingNode::TYPE_NODE:              parts.push_back("\"type\": \"node\"");              break;
        case WingNode::TYPE_LINEAR_FLOAT:      parts.push_back("\"type\": \"linear float\"");      break;
        case WingNode::TYPE_LOGARITHMIC_FLOAT: parts.push_back("\"type\": \"log float\""); break;
        case WingNode::TYPE_FADER_LEVEL:       parts.push_back("\"type\": \"fader level\"");       break;
        case WingNode::TYPE_INTEGER:           parts.push_back("\"type\": \"integer\"");           break;
        case WingNode::TYPE_STRING_ENUM:       parts.push_back("\"type\": \"string enum\"");       break;
        case WingNode::TYPE_FLOAT_ENUM:        parts.push_back("\"type\": \"float enum\"");        break;
        case WingNode::TYPE_STRING:            parts.push_back("\"type\": \"string\"");            break;
    }
    switch (def.getUnit()) {
        case WingNode::UNIT_NONE:         /*parts.push_back("\"unit\": \"none\"");*/ break;
        case WingNode::UNIT_DB:           parts.push_back("\"unit\": \"db\"");       break;
        case WingNode::UNIT_PERCENT:      parts.push_back("\"unit\": \"%\"");        break;
        case WingNode::UNIT_MILLISECONDS: parts.push_back("\"unit\": \"ms\"");       break;
        case WingNode::UNIT_HERTZ:        parts.push_back("\"unit\": \"Hz\"");       break;
        case WingNode::UNIT_METERS:       parts.push_back("\"unit\": \"meters\"");   break;
        case WingNode::UNIT_SECONDS:      parts.push_back("\"unit\": \"seconds\"");  break;
        case WingNode::UNIT_OCTAVES:      parts.push_back("\"unit\": \"octaves\"");  break;
    }
    if (def.isReadOnly()) {
        parts.push_back("\"readOnly\": true");
    }

    if (def.getType() == WingNode::TYPE_STRING) {
        parts.push_back(format("\"maxStringLen\": {},", def.maxStringLen));
    } else if (def.getType() == WingNode::TYPE_LINEAR_FLOAT || def.getType() == WingNode::TYPE_LOGARITHMIC_FLOAT) {
        parts.push_back(format("\"minFloat\": {}, ", def.minFloat));
        parts.push_back(format("\"maxFloat\": {}, ", def.maxFloat));
        parts.push_back(format("\"steps\": {}, ", def.steps));
    } else if (def.getType() == WingNode::TYPE_INTEGER) {
        parts.push_back(format("\"minInt\": {}, ", def.minInt));
        parts.push_back(format("\"maxInt\": {}, ", def.maxInt));
    } else if (def.getType() == WingNode::TYPE_STRING_ENUM && !def.stringEnum.empty()) {
        vector<string> parts2;
        string parts2str;

        for (auto item : def.stringEnum) {
            if (!item.longitem.empty())
                parts2.push_back(format("{{ \"item\": \"{}\", \"text\": \"{}\" }}", item.item, item.longitem));
            else
                parts2.push_back(format("{{ \"item\": \"{}\" }}", item.item));
        }

        for (int k = 0; k < parts2.size(); k++) {
            if (k != 0) parts2str += ", ";
            parts2str += parts2[k];
        }
        parts.push_back(format("\"enumOptions\": [ {} ]", parts2str));
    } else if (def.getType() == WingNode::TYPE_FLOAT_ENUM && !def.floatEnum.empty()) {
        vector<string> parts2;
        string parts2str;

        for (auto item : def.stringEnum) {
            if (!item.longitem.empty())
                parts2.push_back(format("{{ \"item\": \"{}\", \"text\": \"{}\" }}", item.item, item.longitem));
            else
                parts2.push_back(format("{{ \"item\": \"{}\" }}", item.item));
        }

        for (int k = 0; k < parts2.size(); k++) {
            if (k != 0) parts2str += ", ";
            parts2str += parts2[k];
        }
        parts.push_back(format("\"enumOptions\": [ {} ]", parts2str));
    }

    schemaFile << "{";
    for (int k = 0; k < parts.size(); k++) {
        if (k != 0) schemaFile << ", ";
        schemaFile << parts[k];
    }
    schemaFile << "}\n";

    dataFile << format("{{ \"{}\", {:10d} }},\n", fullname, def.id);

    for (int child : _nodeParentToChildren[nodeId]) {
        printNode(child, recurs);
    }
}

int
req(int nodeId, WingConsole &console)
{
    int done = 0;

    if (_nodeParentToChildren.find(nodeId) == _nodeParentToChildren.end()) {
        _nodeParentToChildren[nodeId];
        done++;
        console.requestNodeDefinition(nodeId);
        if (done > 100) return done;
    }

    if (done == 0) {
        for (int child : _nodeParentToChildren[nodeId]) {
            auto def = _nodeIdToDef[child];
            if (def.getType() == WingNode::TYPE_NODE) {
                if (_nodeParentToChildren.find(child) == _nodeParentToChildren.end()) {
                    _nodeParentToChildren[def.id];
                    done++;
                    console.requestNodeDefinition(def.id);
                    if (done > 100) return done;
                }
            }
        }
    }
    if (done == 0) {
        for (int child : _nodeParentToChildren[nodeId]) {
            auto def = _nodeIdToDef[child];
            if (def.getType() == WingNode::TYPE_NODE) {
                int v = req(child, console);
                done += v;
                if (v)
                    break;
            }
        }
    }
    return done;
}

void
gotEnd(WingConsole &console) {
}

int
main()
{
    cout << "Discovering Behringer Wing consoles..." << endl;
    auto discovered = WingConsole::discover();
    
    if (discovered.empty()) {
        cerr << "No Behringer Wing discovered found" << endl;
        return 1;
    } else {
        cout << format("Found {} console(s):\n",
                                 discovered.size());
        for (size_t i = 0; i < discovered.size(); i++) {
            cout << format("    {}. {} ({})\n",
                                     i+1,
                                     discovered[i].name,
                                     discovered[i].ip);
        }
    }

    cout
        << "Connecting to Behringer Wing console "
        << discovered[0].name << endl;

    auto console = WingConsole::connect(discovered[0].ip);
    console.onRequestEnd = [&]() {
        static int done = 1;
        static int ends = 0;
        ends++;
        if (ends == done) {
            int v = req(0, console);
            done += v;
            if (v == 0) {
                cout << "\rSchema retreived. Writing files.\n";
                cout << endl;
                printNode(0, true);
                schemaFile.close();
                dataFile.close();

                cout << "wing-schema.jsonl" << endl;
                cout << "wing-schema.cpp" << endl;
                cout << endl;
                cout << "Done." << endl;

                exit(0);
            }
        }
    };
    console.onNodeDefinition = [](NodeDefinition node) {
        _nodeIdToDef[node.id] = node;
        _nodeParentToChildren[node.parentId].push_back(node.id);
        cout << format("\rReceived {} properties",
                                 _nodeIdToDef.size());
        cout.flush();
    };

    schemaFile.open("wing-schema.jsonl", ios_base::trunc);
    dataFile.open("wing-schema.cpp", ios_base::trunc);

    time_t end_time = system_clock::to_time_t(system_clock::now());
    dataFile
        << "// Generated by wingschema from a Behringer Wing,\n"
        << format("// model {}, firmware {}, on {}",
                       discovered[0].model,
                       discovered[0].firmware,
                       ctime(&end_time))
        << "//\n"
        << "// https://github.com/dannydulai/libwing\n"
        << "//\n"
        << "// This file is included by WingNode.cpp while compiling libwing.\n"
        << "//\n"
        << endl; 

    console.requestNodeDefinition(0);
    console.read();

    return 0;
}
