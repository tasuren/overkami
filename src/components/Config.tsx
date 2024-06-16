import { Component } from "solid-js";

import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger
} from "~/components/ui/dialog";
import { Button } from "./ui/button";
import { HiOutlineCog6Tooth } from "solid-icons/hi";

export const Config: Component<{ className: string }> = (props) => {
    return (
        <>
            <Dialog>
                <DialogTrigger as={Button<"button">} class={props.className}>
                    <HiOutlineCog6Tooth class="scale-150" />
                </DialogTrigger>

                <DialogContent class="rounded-lg w-3/4 h-3/4">
                    <DialogHeader>
                        <DialogTitle>Are you sure absolutely sure?</DialogTitle>
                        <DialogDescription>
                            This action cannot be undone. This will permanently
                            delete your account and remove your data from our
                            servers.
                        </DialogDescription>
                    </DialogHeader>
                </DialogContent>
            </Dialog>
        </>
    );
};
