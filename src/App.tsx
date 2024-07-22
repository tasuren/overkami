import { PreferenceProvider } from "./lib/preference";
import { TitleBar } from "./components/TitleBar";
import { ScreenProvider } from "./screen";

export default function App() {
    return (
        <ScreenProvider>
            <PreferenceProvider>
                <div class="w-screen h-screen flex flex-col bg-light dark:bg-dark">
                    <TitleBar />

                    <div class="w-full flex-1"></div>
                </div>
            </PreferenceProvider>
        </ScreenProvider>
    );
}
