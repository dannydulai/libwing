#include <cstring>
#include <iostream>

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
    console.onNodeData = [&](auto id, auto data) {
        cout
            << NodeDefinition::nodeIdToName(id)
            << " = "
            << data.getString() << endl;
    };

    cout << "Monitoring for changes..." << endl;
    console.read();
    cout << "... device disconnected." << endl;

    return 0;
}
