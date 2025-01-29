#include <cstring>
#include <iostream>

#include "WingConsole.h"

using namespace std;

int
main()
{
    printf("Discovering Behringer Wing consoles...\n");
    auto discovered = WingConsole::scan();
    
    if (discovered.empty()) {
        printf("No Behringer Wing consoles found.\n");
        return 1;
    } else {
        printf("Found %zu consoles(s):\n", discovered.size());
        for (size_t i = 0; i < discovered.size(); i++) {
            printf("    %zu. %s (%s)\n", i+1, discovered[i].name.c_str(), discovered[i].ip.c_str());
        }
        fflush(stdout);
    }

    printf("Connecting to Behringer Wing console %s\n", discovered[0].name.c_str());

    auto console = WingConsole::connect(discovered[0].ip);
    console.onNodeData = [&](auto id, auto data) {
        string name = NodeDefinition::nodeIdToName(id);
        if (name.empty()) {
            printf("<UnknownId:0x%08x> = %s\n", id, data.getString().c_str());
        } else {
            printf("%s = %s\n", name.c_str(), data.getString().c_str());
        }
    };

    cout << "Monitoring for changes..." << endl;
    console.read();
    cout << "... device disconnected." << endl;

    return 0;
}
