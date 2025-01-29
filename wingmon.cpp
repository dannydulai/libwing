#include <cstring>
#include <iostream>
#include <fmt/core.h>

#include "WingConsole.h"

using namespace std;

int
main()
{
    cout << "Discovering Behringer Wing consoles..." << endl;
    auto discovered = WingConsole::scan();
    
    if (discovered.empty()) {
        cerr << "No Behringer Wing consoles found" << endl;
        return 1;
    } else {
        cout << fmt::format("Found {} console(s):\n",
                            discovered.size());
        for (size_t i = 0; i < discovered.size(); i++) {
            cout << fmt::format("    {}. {} ({})\n",
                                i+1,
                                discovered[i].name,
                                discovered[i].ip);
        }
    }

    cout
        << "Connecting to Behringer Wing console "
        << discovered[0].name << endl;

    auto console = WingConsole::connect(discovered[0].ip);
    console.onNodeData = [&](auto id, auto data) {
        string name = NodeDefinition::nodeIdToName(id);
        if (name.empty()) name = fmt::format("<UnknownId:0x{:08x}>", id);
        cout << fmt::format("{} = {}", name, data.getString()) << endl;
    };

    cout << "Monitoring for changes..." << endl;
    console.read();
    cout << "... device disconnected." << endl;

    return 0;
}
