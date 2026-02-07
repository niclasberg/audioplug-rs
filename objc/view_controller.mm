#import <AudioToolbox/AudioToolbox.h>
#import <AVFoundation/AVFoundation.h>
#import <CoreAudioKit/CoreAudioKit.h>
#include <memory>

typedef struct view_controller view_controller_t;

extern "C" view_controller_t* AUV3_create_view_controller();
extern "C" void AUV3_destroy_view_controller(view_controller_t*);
extern "C" NSView* AUV3_create_view(view_controller_t*);
extern "C" AUAudioUnit* AUV3_create_audio_unit(view_controller_t*, AudioComponentDescription desc, NSError ** error);
extern "C" CGSize AUV3_preferred_content_size(view_controller_t*);
extern "C" void AUV3_view_did_layout_subviews(view_controller_t*);

struct Deleter {
	void operator()(view_controller_t* viewController) const {
		AUV3_destroy_view_controller(viewController);
	}
};

@interface AUDIOPLUG_VIEW_CONTROLLER_NAME : AUViewController<AUAudioUnitFactory>
@end
@implementation AUDIOPLUG_VIEW_CONTROLLER_NAME
{
    std::unique_ptr<view_controller_t, Deleter> viewController;
}

- (instancetype) initWithNibName: (nullable NSString*) nib bundle: (nullable NSBundle*) bndl { 
	NSLog(@"[MyAU] Loading view controller");
	self = [super initWithNibName: nib bundle: bndl]; 
	view_controller_t* vc = AUV3_create_view_controller();
	viewController.reset(vc); 
	return self; 
}
- (void) loadView { 
	NSLog(@"[MyAU] Loading view");
	self.view = AUV3_create_view(viewController.get());
}
- (AUAudioUnit *) createAudioUnitWithComponentDescription: (AudioComponentDescription) desc error: (NSError **) error { 
	NSLog(@"[MyAU] Creating audio unit");
	return AUV3_create_audio_unit(viewController.get(), desc, error); 
}
- (CGSize) preferredContentSize  { 
	return AUV3_preferred_content_size(viewController.get());
}

//- (void) viewDidLayoutSubviews   { AUV3_view_did_layout_subviews(viewController.get());  }
//- (void) viewDidLayout           { AUV3_view_did_layout_subviews(viewController.get()); }
//- (void) didReceiveMemoryWarning { cpp->didReceiveMemoryWarning(); }
@end
