import {
    ColorModeProvider,
    ColorModeScript,
    createLocalStorageManager
} from "@kobalte/core";
import { TitleBar } from "./components/TitleBar";
import { ScreenProvider } from "./screen";
import { Separator } from "./components/ui/separator";

function App() {
    const storageManager = createLocalStorageManager("vite-ui-theme");

    return (
        <ScreenProvider>
            <ColorModeScript storageType={storageManager.type} />
            <ColorModeProvider storageManager={storageManager}>
                <TitleBar />

                <div class="w-screen h-full">あ</div>
            </ColorModeProvider>
        </ScreenProvider>
    );
}

export default App;
